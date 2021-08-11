use crate::{graphics_state::GraphicsState, vulkano_layer::VulkanoLayer};
use clockwork_core::{
    clockwork::{CallbackSubstate, ClockworkState},
    prelude::Mechanism,
};
use log::*;
use main_loop::{
    prelude::{Event, Window},
    state::MainLoopState,
};
use std::{sync::Arc, time::Duration};
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    device::{Device, DeviceExtensions, Queue},
    format::Format,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract},
    image::{AttachmentImage, ImageUsage, SwapchainImage},
    instance::{Instance, PhysicalDevice},
    pipeline::viewport::Viewport,
    swapchain::{
        self, AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface,
        SurfaceTransform, Swapchain,
    },
    sync::{self, FlushError, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::window::WindowBuilder;

struct PrivateState {
    swapchain: Arc<Swapchain<Window>>,
    surface: Arc<Surface<Window>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swapchain: bool,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    queue: Arc<Queue>,
}

pub struct VulkanoGraphics<S>
where
    S: ClockworkState,
{
    layers: Vec<Box<dyn VulkanoLayer<S>>>,
    graphics_state: Option<GraphicsState>,
    mechanism_state: Option<PrivateState>,
}

impl<S> VulkanoGraphics<S>
where
    S: ClockworkState,
{
    pub fn builder() -> VulkanoGraphicsBuilder<S> {
        Default::default()
    }
}

pub struct VulkanoGraphicsBuilder<S>(Vec<Box<dyn VulkanoLayer<S>>>)
where
    S: ClockworkState;

impl<S> VulkanoGraphicsBuilder<S>
where
    S: ClockworkState,
{
    pub fn with_layer(mut self, layer: impl VulkanoLayer<S> + 'static) -> Self {
        self.0.push(Box::new(layer));
        self
    }

    pub fn build(self) -> VulkanoGraphics<S> {
        let Self(layers) = self;
        VulkanoGraphics {
            layers,
            graphics_state: None,
            mechanism_state: None,
        }
    }
}

impl<S> Default for VulkanoGraphicsBuilder<S>
where
    S: ClockworkState,
{
    fn default() -> Self {
        VulkanoGraphicsBuilder(Default::default())
    }
}

impl<S> Mechanism<S, Event> for VulkanoGraphics<S>
where
    S: CallbackSubstate<MainLoopState>,
{
    fn name(&self) -> &'static str {
        "Vulkano Graphics"
    }

    fn clink(&mut self, state: &mut S, event: Event) {
        match (event, self) {
            (
                Event::Draw(_),
                Self {
                    layers,
                    graphics_state: Some(graphics_state),
                    mechanism_state:
                        Some(PrivateState {
                            swapchain,
                            surface,
                            previous_frame_end,
                            recreate_swapchain,
                            framebuffers,
                            queue,
                        }),
                },
            ) => {
                /* ---- LOCKING VULKAN STATE ---- */
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                /* ---- HANDLING WINDOW RESIZE ---- */
                let (image_num, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            *recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    *recreate_swapchain = true;
                }

                if *recreate_swapchain {
                    todo!()
                }

                /* ---- DRAWING ---- */
                /* -- BUILDING COMMAND BUFFER --  */
                let command_buffer = {
                    let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
                        graphics_state.device.clone(),
                        queue.family(),
                    )
                    .unwrap();
                    builder
                        .begin_render_pass(
                            framebuffers[image_num].clone(),
                            vulkano::command_buffer::SubpassContents::Inline,
                            vec![[0.0, 0.0, 0.0].into(), 1f32.into()],
                        )
                        .unwrap();

                    for layer in layers {
                        layer.draw(state, graphics_state, &mut builder);
                    }

                    builder.end_render_pass().unwrap();
                    builder
                        .build()
                        .expect("Failed to construct Vulkan command buffer")
                };

                /* -- SUBMITTING COMMAND BUFFER -- */
                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();
                *previous_frame_end = match future {
                    Ok(future) => Some(future.boxed()),
                    Err(FlushError::OutOfDate) => {
                        *recreate_swapchain = true;
                        Some(sync::now(graphics_state.device.clone()).boxed())
                    }
                    Err(e) => panic!("{}", e),
                };
            }
            (Event::Initialization, vulkano_graphics)
                if vulkano_graphics.graphics_state.is_none()
                    && vulkano_graphics.mechanism_state.is_none() =>
            {
                info!("Initializing Vulkano Graphics");

                /* ---- INSTANCE, SURFACE, GPU ---- */
                trace!("Creating Vulkan Instance");
                let instance = Instance::new(
                    None,
                    &vulkano_win::required_extensions(),
                    None,
                ).expect("Failed to create Vulkan instance\nCheck if Vulkan runtime is installed, and, if not, install it from https://vulkan.lunarg.com/sdk/home");

                let surface = {
                    let mut surface = None;
                    state.callback_substate_mut(|MainLoopState(event_loop)| {
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
                let queue_family = physical_device.queue_families().find(|&queue| {
                    queue.supports_graphics()
                        && surface.is_supported(queue).unwrap_or(false)
                }).expect("Failed to find an appropriate queue family for the current physical device");

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

                let mut dynamic_state = DynamicState {
                    line_width: None,
                    viewports: None,
                    scissors: None,
                    compare_mask: None,
                    write_mask: None,
                    reference: None,
                };
                let framebuffers = window_size_dependent_setup(
                    &images,
                    &depth_buffer,
                    render_pass.clone(),
                    &mut dynamic_state,
                );
                let recreate_swapchain = false;
                let previous_frame_end = Some(sync::now(device.clone()).boxed());
                let layers: Vec<_> = vulkano_graphics.layers.drain(..).collect();

                /* ---- WRITING INTERNAL STATE ---- */
                *vulkano_graphics = Self {
                    layers,
                    graphics_state: Some(GraphicsState {
                        dynamic_state,
                        render_pass,
                        device,
                    }),
                    mechanism_state: Some(PrivateState {
                        swapchain,
                        surface,
                        previous_frame_end,
                        recreate_swapchain,
                        framebuffers,
                        queue,
                    }),
                };
                info!("Done initializing Vulkano Graphics")
            }
            _ => unreachable!(),
        }
    }

    fn handled_events(&self) -> Option<&'static [Event]> {
        Some(&[Event::Initialization, Event::Draw(Duration::ZERO)])
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    depth_image: &Arc<AttachmentImage>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
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
