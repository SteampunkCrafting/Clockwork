use clockwork_core::clockwork::{CallbackSubstate, ClockworkState};
use log::{debug, info, trace};
use main_loop::{prelude::Window, state::MainLoopState};
use std::sync::Arc;
use vulkano::{
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageUsage, SwapchainImage},
    instance::Instance,
    render_pass::{Framebuffer, FramebufferAbstract, RenderPass},
    swapchain::{Surface, Swapchain},
    sync::{self, GpuFuture},
    Version,
};
use vulkano_win::VkSurfaceBuild;
use winit::window::WindowBuilder;

pub struct GraphicsState {
    pub surface: Arc<Surface<Window>>,
    pub render_pass: Arc<RenderPass>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

pub(crate) struct InternalMechanismState {
    pub swapchain: Arc<Swapchain<Window>>,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub recreate_swapchain: bool,
    pub framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}

pub trait StateRequirements:
    CallbackSubstate<MainLoopState> + CallbackSubstate<Option<GraphicsState>> + ClockworkState
{
}
impl<T> StateRequirements for T where
    T: CallbackSubstate<MainLoopState> + CallbackSubstate<Option<GraphicsState>> + ClockworkState
{
}

pub(crate) fn init_vulkano<S>(engine_state: &S) -> (InternalMechanismState, GraphicsState)
where
    S: StateRequirements,
{
    /* ---- INSTANCE, SURFACE, GPU ---- */
    trace!("Creating Vulkan Instance");
    let instance = Instance::new(
        None,
        Version::V1_1,
        &vulkano_win::required_extensions(),
        None,
    ).expect("Failed to create Vulkan instance\nCheck if Vulkan runtime is installed, and, if not, install it from https://vulkan.lunarg.com/sdk/home");

    let surface = {
        let mut surface = None;
        CallbackSubstate::callback_substate(engine_state, |MainLoopState(event_loop)| {
            trace!("Getting Winit Event Loop from shared state");
            let event_loop = event_loop
                .as_deref()
                .expect("Missing event loop during initialization");

            trace!("Instantiating window and surface");
            surface = Some(
                WindowBuilder::new()
                    .build_vk_surface(event_loop, instance.clone())
                    .expect("Failed to build surface"),
            );
        });
        surface.unwrap()
    };

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
        vulkano::single_pass_renderpass!(device.clone(),
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
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .unwrap(),
    );
    let framebuffers = window_size_dependent_setup(&images, render_pass.clone());

    /* ---- WRITING INTERNAL STATE ---- */
    (
        InternalMechanismState {
            swapchain: swapchain.clone(),
            previous_frame_end: Some(sync::now(device.clone()).boxed()),
            recreate_swapchain: false,
            framebuffers,
        },
        GraphicsState {
            surface: swapchain.surface().clone(),
            render_pass,
            device,
            queue,
        },
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
