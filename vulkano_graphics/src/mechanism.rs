use clockwork_core::{clockwork::Substate, prelude::Mechanism};
use log::*;
use main_loop::{
    prelude::{Event, Window},
    state::IOState,
};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    device::{Device, DeviceExtensions, Queue},
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{ImageUsage, SwapchainImage},
    impl_vertex,
    instance::{Instance, PhysicalDevice},
    pipeline::{
        vertex::OneVertexOneInstanceDefinition, viewport::Viewport, GraphicsPipeline,
        GraphicsPipelineAbstract,
    },
    single_pass_renderpass,
    swapchain::{
        self, AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface,
        SurfaceTransform, Swapchain, SwapchainCreationError,
    },
    sync::{self, FlushError, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::window::WindowBuilder;

use crate::vulkano_layer::VulkanoLayer;

struct LocalState {
    dynamic_state: DynamicState,
    swapchain: Arc<Swapchain<Window>>,
    surface: Arc<Surface<Window>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swapchain: bool,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    device: Arc<Device>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    queue: Arc<Queue>,
}

pub struct VulkanoGraphics {
    layers: Vec<VulkanoLayer>,
    state: Option<LocalState>,
}

impl<S> Mechanism<S, Event> for VulkanoGraphics
where
    S: Substate<IOState>,
{
    fn name(&self) -> &'static str {
        "Vulkano Graphics"
    }

    fn clink(&mut self, state: &mut S, event: Event) {
        match (event, self) {
            (
                Event::Draw(_),
                Self {
                    state:
                        Some(LocalState {
                            dynamic_state,
                            swapchain,
                            surface,
                            previous_frame_end,
                            recreate_swapchain,
                            framebuffers,
                            render_pass,
                            device,
                            pipeline,
                            queue,
                        }),
                    ..
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
                    let dimensions: [u32; 2] = surface.window().inner_size().into();
                    let (new_swapchain, new_images) =
                        match swapchain.recreate_with_dimensions(dimensions) {
                            Ok(r) => r,
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };
                    *swapchain = new_swapchain;
                    *framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        dynamic_state,
                    );
                    *recreate_swapchain = false;
                }

                /* ---- DRAWING ---- */
                /* -- CREATING BUFFERS --  */
                let triangle_vertex_buffer = {
                    CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        [
                            Vertex {
                                position: [-0.5, -0.25],
                            },
                            Vertex {
                                position: [0.0, 0.5],
                            },
                            Vertex {
                                position: [0.25, -0.1],
                            },
                        ]
                        .iter()
                        .cloned(),
                    )
                    .unwrap()
                };

                let instance_data_buffer = {
                    let rows = 10;
                    let cols = 10;
                    let n_instances = rows * cols;
                    let mut data = Vec::new();
                    for c in 0..cols {
                        for r in 0..rows {
                            let half_cell_w = 0.5 / cols as f32;
                            let half_cell_h = 0.5 / rows as f32;
                            let x = half_cell_w + (c as f32 / cols as f32) * 2.0 - 1.0;
                            let y = half_cell_h + (r as f32 / rows as f32) * 2.0 - 1.0;
                            let position_offset = [x, y];
                            let scale =
                                (2.0 / rows as f32) * (c * rows + r) as f32 / n_instances as f32;
                            data.push(InstanceData {
                                position_offset,
                                scale,
                            });
                        }
                    }
                    CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        data.iter().cloned(),
                    )
                    .unwrap()
                };

                /* -- BUILDING COMMAND BUFFER --  */
                let command_buffer = {
                    let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
                        device.clone(),
                        queue.family(),
                    )
                    .unwrap();

                    builder
                        .begin_render_pass(
                            framebuffers[image_num].clone(),
                            vulkano::command_buffer::SubpassContents::Inline,
                            vec![[0.0, 0.0, 0.0].into()],
                        )
                        .unwrap()
                        .draw(
                            pipeline.clone(),
                            &dynamic_state,
                            vec![triangle_vertex_buffer.clone(), instance_data_buffer.clone()],
                            (),
                            (),
                        )
                        .unwrap()
                        .end_render_pass()
                        .unwrap();

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
                        Some(sync::now(device.clone()).boxed())
                    }
                    Err(e) => panic!("{}", e),
                };
            }
            (Event::Initialization, vulkano_graphics @ Self { state: None, .. }) => {
                info!("Initializing Vulkano Graphics");

                /* ---- INSTANCE, SURFACE, GPU ---- */
                trace!("Getting Winit Event Loop from shared state");
                let event_loop = Substate::<IOState>::substate(state)
                    .event_loop
                    .as_deref()
                    .expect("Missing event loop during initialization");

                trace!("Creating Vulkan Instance");
                let instance = Instance::new(
                    None,
                    &vulkano_win::required_extensions(),
                    None,
                ).expect("Failed to create Vulkan instance\nCheck if Vulkan runtime is installed, and, if not, install it from https://vulkan.lunarg.com/sdk/home");

                trace!("Instantiating window and surface");
                let surface = WindowBuilder::new()
                    .build_vk_surface(event_loop, instance.clone())
                    .expect("Failed to build surface");

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

                /* ---- SWAPCHAIN, IMAGES ---- */
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

                /* ---- STUFF TO ALLOCATE SOMEWHERE ELSE ---- */

                let vs = vs::Shader::load(device.clone()).unwrap();
                let fs = fs::Shader::load(device.clone()).unwrap();

                let render_pass = Arc::new(
                    single_pass_renderpass!(
                        device.clone(),
                        attachments: {
                            color: {
                                load: Clear,
                                store: Store,
                                format: swapchain.format(),
                                samples: 1,
                            }
                        },
                        pass: {
                            color: [color],
                            depth_stencil: {}
                        }
                    )
                    .unwrap(),
                );

                let pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync> = Arc::new(
                    GraphicsPipeline::start()
                        .vertex_input(OneVertexOneInstanceDefinition::<Vertex, InstanceData>::new())
                        .vertex_shader(vs.main_entry_point(), ())
                        .triangle_list()
                        .viewports_dynamic_scissors_irrelevant(1)
                        .fragment_shader(fs.main_entry_point(), ())
                        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                        .build(device.clone())
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
                let framebuffers =
                    window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);
                let recreate_swapchain = false;
                let previous_frame_end = Some(sync::now(device.clone()).boxed());

                /* ---- WRITING INTERNAL STATE ---- */
                *vulkano_graphics = Self {
                    layers: vec![],
                    state: Some(LocalState {
                        dynamic_state,
                        swapchain,
                        surface,
                        previous_frame_end,
                        recreate_swapchain,
                        framebuffers,
                        render_pass,
                        device,
                        pipeline,
                        queue,
                    }),
                };
                info!("Done initializing Vulkano Graphics")
            }
            _ => unreachable!(),
        }
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
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
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
impl_vertex!(Vertex, position);

#[derive(Default, Debug, Clone)]
struct InstanceData {
    position_offset: [f32; 2],
    scale: f32,
}
impl_vertex!(InstanceData, position_offset, scale);

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
                #version 450
                // The triangle vertex positions.
                layout(location = 0) in vec2 position;
                // The per-instance data.
                layout(location = 1) in vec2 position_offset;
                layout(location = 2) in float scale;
                void main() {
                    // Apply the scale and offset for the instance.
                    gl_Position = vec4(position * scale + position_offset, 0.0, 1.0);
                }
            "
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
                #version 450
                layout(location = 0) out vec4 f_color;
                void main() {
                    f_color = vec4(1.0, 0.0, 0.0, 1.0);
                }
            "
    }
}
