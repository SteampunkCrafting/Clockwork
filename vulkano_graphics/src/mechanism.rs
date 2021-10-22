use crate::{
    state::{
        init_vulkano, window_size_dependent_setup, GraphicsState, InternalMechanismState,
        StateRequirements,
    },
    vulkano_layer::VulkanoLayer,
};
use clockwork_core::{
    clockwork::{CallbackSubstate, ClockworkState},
    prelude::Mechanism,
};
use log::*;
use main_loop::prelude::Event;
use std::time::Duration;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    format::Format,
    image::AttachmentImage,
    pipeline::viewport::Viewport,
    swapchain::{self, AcquireError, SwapchainCreationError},
    sync::{self, FlushError, GpuFuture},
};
use winit::dpi::PhysicalSize;

pub struct VulkanoGraphics<S>
where
    S: ClockworkState,
{
    inner: Option<InternalMechanismState>,
    layers: Vec<Box<dyn VulkanoLayer<S>>>,
}

impl<S> VulkanoGraphics<S>
where
    S: StateRequirements,
{
    pub fn builder() -> VulkanoGraphicsBuilder<S> {
        Default::default()
    }
}

pub struct VulkanoGraphicsBuilder<S>(Vec<Box<dyn VulkanoLayer<S>>>)
where
    S: StateRequirements;

impl<S> VulkanoGraphicsBuilder<S>
where
    S: StateRequirements,
{
    pub fn with_layer(mut self, layer: impl VulkanoLayer<S> + 'static) -> Self {
        self.0.push(Box::new(layer));
        self
    }

    pub fn build(self) -> VulkanoGraphics<S> {
        let Self(layers) = self;
        VulkanoGraphics {
            inner: None,
            layers,
        }
    }
}

impl<S> Default for VulkanoGraphicsBuilder<S>
where
    S: StateRequirements,
{
    fn default() -> Self {
        VulkanoGraphicsBuilder(Default::default())
    }
}

impl<S> Mechanism<S, Event> for VulkanoGraphics<S>
where
    S: StateRequirements,
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
                    inner: Some(internal_state),
                },
            ) => draw(state, layers, internal_state),
            (
                Event::Initialization,
                Self {
                    inner: inner @ None,
                    ..
                },
            ) => {
                info!("Initializing Vulkano Graphics");
                let (internal, graphics_state) = init_vulkano(state);
                *inner = Some(internal);
                CallbackSubstate::<Option<GraphicsState>>::callback_substate_mut(state, |gs| {
                    *gs = Some(graphics_state);
                });
                info!("Done initializing Vulkano Graphics");
            }
            _ => unreachable!(),
        }
    }

    fn handled_events(&self) -> Option<&'static [Event]> {
        Some(&[Event::Initialization, Event::Draw(Duration::ZERO)])
    }
}

/// Draws on the window by means of activating VulkanoLayers
fn draw<S>(
    state: &S,
    layers: &mut Vec<Box<dyn VulkanoLayer<S>>>,
    InternalMechanismState {
        swapchain,
        previous_frame_end,
        recreate_swapchain,
        framebuffers,
    }: &mut InternalMechanismState,
) where
    S: StateRequirements,
{
    CallbackSubstate::<Option<GraphicsState>>::callback_substate(state, |graphics_state| {
        let graphics_state = graphics_state.as_ref().unwrap();
        /* ---- GARBAGE COLLECTING ---- */
        previous_frame_end.as_mut().unwrap().cleanup_finished();

        /* ---- HANDLING WINDOW RESIZE ---- */
        if *recreate_swapchain {
            // Get the new dimensions of the window.
            let dimensions: [u32; 2] = swapchain.surface().window().inner_size().into();
            let (new_swapchain, new_images) =
                match swapchain.recreate().dimensions(dimensions).build() {
                    Ok(r) => r,
                    Err(SwapchainCreationError::UnsupportedDimensions) => return,
                    Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                };
            let new_images = new_images
                .into_iter()
                .map(|image| {
                    (
                        image,
                        AttachmentImage::transient(
                            graphics_state.device.clone(),
                            dimensions,
                            Format::D16_UNORM,
                        )
                        .unwrap(),
                    )
                })
                .collect::<Vec<_>>();

            *swapchain = new_swapchain;
            *framebuffers =
                window_size_dependent_setup(&new_images, graphics_state.render_pass.clone());
            *recreate_swapchain = false;
        }

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
            *recreate_swapchain = true
        }

        /* ---- DRAWING ---- */
        /* -- BUILDING COMMAND BUFFER --  */
        let command_buffer = {
            let mut builder = AutoCommandBufferBuilder::primary(
                graphics_state.device.clone(),
                graphics_state.queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();
            builder
                .set_viewport(
                    0,
                    [{
                        let PhysicalSize { width, height } =
                            graphics_state.surface.window().inner_size();
                        Viewport {
                            origin: [0.0, height as f32],
                            dimensions: [width as f32, -(height as f32)],
                            depth_range: 0.0..1.0,
                        }
                    }],
                )
                .begin_render_pass(
                    framebuffers[image_num].clone(),
                    vulkano::command_buffer::SubpassContents::Inline,
                    vec![[0.0, 0.0, 0.0].into(), 1f32.into()],
                )
                .unwrap();

            for layer in layers {
                layer.draw(state, &mut builder);
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
            .then_execute(graphics_state.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(graphics_state.queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();
        *previous_frame_end = match future {
            Ok(future) => Some(future.boxed()),
            Err(FlushError::OutOfDate) => {
                *recreate_swapchain = true;
                Some(sync::now(graphics_state.device.clone()).boxed())
            }
            Err(e) => panic!("{}", e),
        };
    });
}
