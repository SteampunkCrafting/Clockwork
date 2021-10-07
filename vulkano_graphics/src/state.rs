use clockwork_core::clockwork::{CallbackSubstate, ClockworkState};
use log::{debug, info, trace};
use main_loop::{prelude::Window, state::MainLoopState};
use std::sync::Arc;
use vulkano::{
    command_buffer::DynamicState,
    device::{Device, DeviceExtensions, Queue},
    format::Format,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract},
    image::{AttachmentImage, ImageUsage, SwapchainImage},
    instance::{Instance, PhysicalDevice},
    pipeline::viewport::Viewport,
    swapchain::{ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform, Swapchain},
    sync::{self, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::window::WindowBuilder;

pub struct GraphicsState {
    pub dynamic_state: DynamicState,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

pub(crate) struct InternalMechanismState {
    pub swapchain: Arc<Swapchain<Window>>,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub recreate_swapchain: bool,
    pub framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}

pub trait StateRequirements: CallbackSubstate<MainLoopState> + ClockworkState {}
impl<T> StateRequirements for T where T: CallbackSubstate<MainLoopState> + ClockworkState {}

pub(crate) fn init_vulkano<S>(engine_state: &S) -> (InternalMechanismState, GraphicsState)
where
    S: StateRequirements,
{
    /* ---- INSTANCE, SURFACE, GPU ---- */
    trace!("Creating Vulkan Instance");
    let instance = Instance::new(
        None,
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
            .map(|d| d.name().to_string())
            .collect::<Vec<_>>()
    );
    let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();

    info!(
        "Rendering through device {:?} of type {:?}",
        physical_device.name().clone(),
        physical_device.ty()
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
        &device_ext,
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
        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            ImageUsage::color_attachment(),
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        )
        .unwrap()
    };
    let depth_buffer = AttachmentImage::transient(
        device.clone(),
        surface.window().inner_size().into(),
        Format::D16Unorm,
    )
    .unwrap();

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
                    format: Format::D16Unorm,
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

    let mut dynamic_state = DynamicState::none();
    let framebuffers = window_size_dependent_setup(
        &images,
        &depth_buffer,
        render_pass.clone(),
        &mut dynamic_state,
    );

    /* ---- WRITING INTERNAL STATE ---- */
    (
        InternalMechanismState {
            swapchain,
            previous_frame_end: Some(sync::now(device.clone()).boxed()),
            recreate_swapchain: false,
            framebuffers,
        },
        GraphicsState {
            dynamic_state,
            render_pass,
            device,
            queue,
        },
    )
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    depth_image: &Arc<AttachmentImage>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, dimensions[1] as f32],
        dimensions: [dimensions[0] as f32, -(dimensions[1] as f32)],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .add(depth_image.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
