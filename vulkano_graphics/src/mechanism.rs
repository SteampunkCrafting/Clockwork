use crate::{
    state::{
        init_vulkano, window_size_dependent_setup, GraphicsInitState, GraphicsState, GuiState,
        InternalMechanismState, StateRequirements,
    },
    vulkano_layer::VulkanoLayer,
};
use kernel::{
    abstract_runtime::{ClockworkState, EngineState},
    util::{derive_builder::Builder, sync::WriteLock},
};
use kernel::{
    prelude::StandardEvent,
    standard_runtime::{StandardEventSuperset, StandardMechanism},
};
use main_loop::state::InitWinitState;
use std::marker::PhantomData;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents},
    format::Format,
    image::AttachmentImage,
    pipeline::viewport::Viewport,
    swapchain::{self, AcquireError, SwapchainCreationError},
    sync::{self, FlushError, GpuFuture},
};
use winit::dpi::PhysicalSize;

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct VulkanoGraphics<S, E>
where
    S: ClockworkState,
    E: StandardEventSuperset,
{
    #[builder(setter(skip))]
    inner: Option<InternalMechanismState>,

    #[builder(private, setter(name = "__layers", into = "false"))]
    layers: Vec<Box<dyn VulkanoLayer<S>>>,

    #[builder(setter(skip))]
    phantom_data: PhantomData<E>,
}

impl<S, E> VulkanoGraphics<S, E>
where
    S: StateRequirements<E>,
    E: StandardEventSuperset,
{
    pub fn builder() -> VulkanoGraphicsBuilder<S, E> {
        Default::default()
    }
}

impl<S, E> VulkanoGraphicsBuilder<S, E>
where
    S: StateRequirements<E>,
    E: StandardEventSuperset,
{
    pub fn add_layer(mut self, layer: impl VulkanoLayer<S> + 'static) -> Self {
        self.layers
            .get_or_insert(Default::default())
            .push(Box::new(layer));
        self
    }
}

impl<S, E> StandardMechanism<S> for VulkanoGraphics<S, E>
where
    S: StateRequirements<E>,
    E: StandardEventSuperset,
{
    fn initialization(&mut self, state: &mut EngineState<S>) {
        let (internal, graphics, gui) = state.start_access().get(|s: &S| init_vulkano(s)).finish();
        let gui = WriteLock::from(gui);
        let _ = self.inner.insert(internal);
        state
            .start_mutate()
            .get_mut(|s: &mut GraphicsInitState| s.initialize(move |_| graphics))
            .get_mut(|s: &mut GuiState| s.initialize(gui.clone()))
            .get_mut(|s: &mut InitWinitState<E>| {
                s.add_event_callback(move |ev| gui.lock_mut().update(ev))
            })
            .finish()
    }

    fn draw(&mut self, state: &mut EngineState<S>) {
        let (layers, inner) = (&mut self.layers, self.inner.as_mut().unwrap());
        draw(state, layers, inner)
    }

    fn handled_events(&self) -> Option<Vec<StandardEvent>> {
        Some(vec![StandardEvent::Initialization, StandardEvent::Draw])
    }

    fn tick(&mut self, _: &mut EngineState<S>) {
        unreachable!()
    }

    fn termination(&mut self, _: &mut EngineState<S>) {
        unreachable!()
    }
}

/// Draws on the window by means of activating VulkanoLayers
fn draw<S, E>(
    state: &mut EngineState<S>,
    layers: &mut Vec<Box<dyn VulkanoLayer<S>>>,
    InternalMechanismState {
        swapchain,
        previous_frame_end,
        recreate_swapchain,
        framebuffers,
    }: &mut InternalMechanismState,
) where
    S: StateRequirements<E>,
    E: StandardEventSuperset,
{
    let GraphicsState {
        target_image_size,
        render_pass,
        device,
        queue,
    } = state
        .start_access()
        .get(|gs: &GraphicsInitState| gs.get_init().clone())
        .finish();

    /* ---- GARBAGE COLLECTING ---- */
    previous_frame_end.as_mut().unwrap().cleanup_finished();

    /* ---- HANDLING WINDOW RESIZE ---- */
    if *recreate_swapchain {
        // Get the new dimensions of the window.
        let dimensions: [u32; 2] = swapchain.surface().window().inner_size().into();
        let (new_swapchain, new_images) = match swapchain.recreate().dimensions(dimensions).build()
        {
            Ok(r) => r,
            Err(SwapchainCreationError::UnsupportedDimensions) => return,
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
        let depth_image =
            AttachmentImage::transient(device.clone(), dimensions, Format::D16_UNORM).unwrap();
        let new_images = new_images
            .into_iter()
            .map(|image| (image, depth_image.clone()))
            .collect::<Vec<_>>();

        *swapchain = new_swapchain;
        *framebuffers = window_size_dependent_setup(&new_images, render_pass.clone());
        *recreate_swapchain = false;

        state.start_mutate().get_mut(|gs: &mut GraphicsInitState| {
            let PhysicalSize { width, height } = swapchain.surface().window().inner_size();
            gs.get_init_mut().target_image_size = [width, height];
        });
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
    let [width, height] = target_image_size;

    /* -- BUILDING COMMAND BUFFER --  */
    let command_buffer = {
        let mut builder = AutoCommandBufferBuilder::primary(
            device.clone(),
            queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        builder
            .set_viewport(
                0,
                [Viewport {
                    origin: [0.0, height as f32],
                    dimensions: [width as f32, -(height as f32)],
                    depth_range: 0.0..1.0,
                }],
            )
            .begin_render_pass(
                framebuffers[image_num].clone(),
                vulkano::command_buffer::SubpassContents::Inline,
                vec![[0.0, 0.0, 0.0].into(), 1f32.into()],
            )
            .unwrap();

        state.start_access().get(|state: &S| {
            layers
                .iter_mut()
                .for_each(|layer| layer.draw(state, &mut builder));
        });

        builder
            .next_subpass(SubpassContents::SecondaryCommandBuffers)
            .unwrap();

        builder
            .execute_commands(
                state
                    .start_mutate()
                    .get_mut(|gui: &mut GuiState| gui.init_draw_on_subpass_image([width, height]))
                    .finish(),
            )
            .unwrap();

        builder.end_render_pass().unwrap();
        builder
            .build()
            .expect("Failed to construct Vulkan command buffer")
    };

    /* -- SUBMITTING COMMAND BUFFER -- */
    *previous_frame_end = match previous_frame_end
        .take()
        .unwrap()
        .join(acquire_future)
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
        .then_signal_fence_and_flush()
    {
        Ok(future) => {
            future.wait(None).unwrap();
            Some(future.boxed())
        }
        Err(FlushError::OutOfDate) => {
            *recreate_swapchain = true;
            Some(sync::now(device.clone()).boxed())
        }
        Err(e) => panic!("{}", e),
    };
}
