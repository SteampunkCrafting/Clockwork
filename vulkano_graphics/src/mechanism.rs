use crate::{
    state::{init_vulkano, GraphicsState, InternalMechanismState, StateRequirements},
    vulkano_layer::VulkanoLayer,
};
use clockwork_core::{clockwork::ClockworkState, prelude::Mechanism};
use log::*;
use main_loop::prelude::Event;
use std::time::Duration;
use vulkano::{
    command_buffer::AutoCommandBufferBuilder,
    swapchain::{self, AcquireError},
    sync::{self, FlushError, GpuFuture},
};

pub struct VulkanoGraphics<S>
where
    S: ClockworkState,
{
    inner: Option<(InternalMechanismState, GraphicsState)>,
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
                    inner: Some((internal_state, graphics_state)),
                },
            ) => draw(state, layers, internal_state, graphics_state),
            (
                Event::Initialization,
                Self {
                    inner: inner @ None,
                    ..
                },
            ) => {
                info!("Initializing Vulkano Graphics");
                *inner = Some(init_vulkano(state));
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
    graphics_state: &mut GraphicsState,
) where
    S: StateRequirements,
{
    /* ---- GARBAGE COLLECTING ---- */
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
            graphics_state.queue.family(),
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
}
