use std::sync::Arc;

use egui_winit_vulkano::Gui;
use kernel::abstract_runtime::EngineState;
use kernel::util::init_state::InitState;
use kernel::util::log::{debug, info, trace};
use kernel::util::sync::WriteLock;
use kernel::{
    abstract_runtime::{ClockworkState, Substate},
    standard_runtime::StandardEventSuperset,
};
use main_loop::{prelude::Window, state::InitWinitState};
use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use vulkano::{
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageUsage, SwapchainImage},
    instance::Instance,
    render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass},
    swapchain::Swapchain,
    sync::{self, GpuFuture},
    Version,
};
use vulkano_win::VkSurfaceBuild;
use winit::{dpi::PhysicalSize, window::WindowBuilder};

#[derive(Clone)]
pub struct GraphicsState {
    pub target_image_size: [u32; 2],
    pub subpass: Subpass,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

pub(crate) struct InternalMechanismState {
    pub swapchain: Arc<Swapchain<Window>>,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub recreate_swapchain: bool,
    pub framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    pub graphics_state: GraphicsState,
}

pub struct GuiState {
    inner: InitState<
        Option<Box<dyn FnMut(egui::CtxRef) + Send>>,
        (Option<Box<dyn FnMut(egui::CtxRef) + Send>>, WriteLock<Gui>),
    >,
}

impl ClockworkState for GuiState {}

impl GuiState {
    /// Updates UI synchronously.
    ///
    /// In order for this method to work, one must call it with the draw call
    /// before the `VulkanoGraphics` starts drawing (or at least with the same
    /// frequency as the draw calls). If this is not an option, consider using
    /// `GuiState::async_ui`.
    ///
    /// **Failing to accomplish the same draw frequency may lead to UI image flickering**
    ///
    /// # Panics
    /// Panics when the method is called before the state initialization happens.
    pub fn immediate_ui(&mut self, callback: impl FnOnce(egui::CtxRef)) {
        let (draw_fn, gui) = self.inner.get_init_mut();
        *draw_fn = None;
        gui.lock_mut().immediate_ui(|gui| callback(gui.context()))
    }

    /// Updates UI asynchronously.
    ///
    /// This method always works, but the callback requires `Send` and `'static`,
    /// which will require additional synchronization logic in order to perform reading from state.
    /// If this is not an option, consider using `GuiState::immediate_ui`.
    pub fn async_ui(&mut self, callback: impl FnMut(egui::CtxRef) + Send + 'static) {
        *match &mut self.inner {
            InitState::Uninit(x) => x,
            InitState::Init((x, _)) => x,
            _ => panic!("Gui state is terminated -- cannot update UI"),
        } = Some(Box::from(callback))
    }

    /// State initialization
    pub(crate) fn initialize(&mut self, gui: WriteLock<Gui>) {
        self.inner.initialize(|draw_fn| (draw_fn, gui))
    }

    /// Draw call
    pub(crate) fn init_draw_on_subpass_image(
        &mut self,
        image_dimensions: [u32; 2],
    ) -> SecondaryAutoCommandBuffer {
        let (draw_fn, gui) = self.inner.get_init_mut();
        let mut gui = gui.lock_mut();
        draw_fn
            .as_mut()
            .map_or((), |draw_fn| gui.immediate_ui(|gui| draw_fn(gui.context())));
        gui.draw_on_subpass_image(image_dimensions)
    }
}

impl Default for GuiState {
    fn default() -> Self {
        Self { inner: None.into() }
    }
}

pub trait StateRequirements<E>
where
    Self: Substate<InitWinitState<E>> + Substate<GuiState> + ClockworkState,
    E: StandardEventSuperset,
{
}
impl<T, E> StateRequirements<E> for T
where
    T: Substate<InitWinitState<E>> + Substate<GuiState> + ClockworkState,
    E: StandardEventSuperset,
{
}

pub(crate) fn init_vulkano<S, E>(engine_state: &EngineState<S>) -> (InternalMechanismState, Gui)
where
    S: StateRequirements<E>,
    E: StandardEventSuperset,
{
    /* ---- INSTANCE, SURFACE, GPU ---- */
    trace!("Creating Vulkan Instance");
    let instance = Instance::new(
        None,
        Version::V1_1,
        &vulkano_win::required_extensions(),
        None,
    ).expect("Failed to create Vulkan instance\nCheck if Vulkan runtime is installed, and, if not, install it from https://vulkan.lunarg.com/sdk/home");

    let surface = engine_state
        .start_access()
        .get(|ml: &InitWinitState<E>| {
            trace!("Instantiating window and surface");
            WindowBuilder::new()
                .build_vk_surface(ml.uninit_event_loop(), instance.clone())
                .expect("Failed to build surface")
        })
        .finish();

    trace!("Getting physical device");
    debug!(
        "Available devices: {:?}",
        PhysicalDevice::enumerate(&instance)
            .map(|d| d.properties().device_name.clone())
            .collect::<Vec<_>>()
    );
    let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();

    info!(
        "Rendering through device {:?} of type {:?}",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    /* ---- LOGICAL DEVICE, QUEUE ---- */
    trace!("Getting queue family");
    let queue_family = physical_device
        .queue_families()
        .find(|&queue| queue.supports_graphics() && surface.is_supported(queue).unwrap_or(false))
        .expect("Failed to find an appropriate queue family for the current physical device");

    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };

    trace!("Creating logical device with extensions: {:#?}", device_ext);
    let (device, mut queues) = Device::new(
        physical_device,
        physical_device.supported_features(),
        &physical_device.required_extensions().union(&device_ext),
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();
    let queue = queues.next().expect("Failed to create queue");

    /* ---- SWAPCHAIN, IMAGES, DEPTH BUFFER ---- */
    let (swapchain, images) = {
        let caps = surface.capabilities(physical_device).unwrap();
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let dimensions: [u32; 2] = surface.window().inner_size().into();
        let (swapchain, images) = Swapchain::start(device.clone(), surface.clone())
            .num_images(caps.min_image_count)
            .format(format)
            .dimensions(dimensions)
            .usage(ImageUsage::color_attachment())
            .sharing_mode(&queue)
            .composite_alpha(alpha)
            .build()
            .unwrap();

        let images = images
            .into_iter()
            .map(|image| {
                (
                    image,
                    AttachmentImage::transient(
                        device.clone(),
                        surface.window().inner_size().into(),
                        Format::D16_UNORM,
                    )
                    .unwrap(),
                )
            })
            .collect::<Vec<_>>();

        (swapchain, images)
    };

    /* ---- STUFF TO ALLOCATE SOMEWHERE ELSE ---- */
    let render_pass = Arc::new(
        vulkano::ordered_passes_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            passes: [
                // Normal render pass
                { color: [color], depth_stencil: {depth}, input: [] },

                // eGUI render pass
                { color: [color], depth_stencil: {}, input: [] }
            ]
        )
        .unwrap(),
    );
    let framebuffers = window_size_dependent_setup(&images, render_pass.clone());

    /* ---- GUI ---- */
    let mut gui = Gui::new_with_subpass(
        surface.clone(),
        queue.clone(),
        Subpass::from(render_pass.clone(), 1).unwrap(),
    );
    gui.immediate_ui(|_| {});

    /* ---- WRITING INTERNAL STATE ---- */
    (
        InternalMechanismState {
            swapchain: swapchain.clone(),
            previous_frame_end: Some(sync::now(device.clone()).boxed()),
            recreate_swapchain: false,
            framebuffers,
            graphics_state: GraphicsState {
                target_image_size: {
                    let PhysicalSize { width, height } = swapchain.surface().window().inner_size();
                    [width, height]
                },
                subpass: Subpass::from(render_pass, 0).unwrap().into(),
                queue: queue.clone(),
                device,
            },
        },
        gui,
    )
}

pub(crate) fn window_size_dependent_setup(
    images: &[(Arc<SwapchainImage<Window>>, Arc<AttachmentImage>)],
    render_pass: Arc<RenderPass>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    images
        .iter()
        .cloned()
        .map(|(image, depth_image)| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(ImageView::new(image).unwrap())
                    .unwrap()
                    .add(ImageView::new(depth_image).unwrap())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect()
}
