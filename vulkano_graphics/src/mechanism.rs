use crate::{
    state::{
        init_vulkano, window_size_dependent_setup, GraphicsState, GuiState, InternalMechanismState,
        StateRequirements,
    },
    vulkano_layer::VulkanoLayer,
};
use kernel::{
    abstract_runtime::{ClockworkState, EngineState},
    util::{derive_builder::Builder, init_state::InitState, sync::WriteLock},
};
use kernel::{
    prelude::StandardEvent,
    standard_runtime::{StandardEventSuperset, StandardMechanism},
};
use main_loop::prelude::{Event, WindowEvent};
use main_loop::state::InitWinitState;
use std::marker::PhantomData;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents},
    format::Format,
    image::AttachmentImage,
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
    inner: InitState<(), InternalMechanismState>,

    #[builder(private, setter(name = "__layers", into = "false"), default)]
    layers: Vec<Box<dyn VulkanoLayer<S>>>,

    #[builder(setter(skip))]
    phantom_data: PhantomData<E>,

    #[builder(setter(skip), default)]
    window_resize: WriteLock<Option<[u32; 2]>>,
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
    pub fn add_layer(mut self, old_layer: impl VulkanoLayer<S> + 'static) -> Self {
        self.layers
            .get_or_insert(Default::default())
            .push(Box::new(old_layer));
        self
    }
}

impl<S, E> StandardMechanism<S> for VulkanoGraphics<S, E>
where
    S: StateRequirements<E>,
    E: StandardEventSuperset,
{
    fn initialization(&mut self, state: &mut EngineState<S>) {
        /* ---- INITIALIZING OWN STATE ---- */
        let (internal, gui) = init_vulkano(state);
        let gui = WriteLock::from(gui);
        self.inner.initialize(|()| internal);
        state
            .start_mutate()
            /* -- INITIALIZING SHARED STATE -- */
            .get_mut(|s: &mut GuiState| s.initialize(gui.clone()))
            /* -- SUBSCRIBING TO WINDOW RESIZE EVENT -- */
            .get_mut(|s: &mut InitWinitState<E>| {
                let mut window_resize = self.window_resize.downgrade_to_user_lock();
                s.add_event_callback(move |ev| match ev {
                    Event::WindowEvent {
                        window_id: _,
                        event: WindowEvent::Resized(PhysicalSize { width, height }),
                    } => *window_resize.lock_mut() = Some([*width, *height]),
                    _ => (),
                });
                s.add_event_callback(move |ev| gui.lock_mut().update(ev));
            })
            .finish();

        /* ---- INITIALIZING LAYER STATES ---- */
        let Self { layers, inner, .. } = self;
        layers
            .iter_mut()
            .for_each(|layer| layer.initialization(state, &inner.get_init().graphics_state))
    }

    fn draw(&mut self, state: &mut EngineState<S>) {
        /* ---- HANDLING POTENTIAL RESIZE ---- */
        let window_resize = self.window_resize.lock_mut().take();
        window_resize.map_or((), |dims| {
            let Self { layers, inner, .. } = self;
            let InternalMechanismState { graphics_state, .. } = inner.get_init_mut();
            graphics_state.target_image_size = dims;
            layers
                .iter_mut()
                .for_each(|layer| layer.window_resize(state, graphics_state))
        });

        /* ---- DRAWING ---- */
        draw(state, &mut self.layers, self.inner.get_init_mut());
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

fn draw<S, E>(
    state: &mut EngineState<S>,
    layers: &mut Vec<Box<dyn VulkanoLayer<S>>>,
    InternalMechanismState {
        swapchain,
        previous_frame_end,
        recreate_swapchain,
        framebuffers,
        graphics_state,
    }: &mut InternalMechanismState,
) where
    S: StateRequirements<E>,
    E: StandardEventSuperset,
{
    let graphics_state
    @
    GraphicsState {
        target_image_size,
        subpass: render_pass,
        device,
        queue,
    } = &graphics_state;
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
        *framebuffers = window_size_dependent_setup(&new_images, render_pass.render_pass().clone());
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
            device.clone(),
            queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        builder
            .begin_render_pass(
                framebuffers[image_num].clone(),
                vulkano::command_buffer::SubpassContents::SecondaryCommandBuffers,
                vec![[0.0, 0.0, 0.0].into(), 1f32.into()],
            )
            .unwrap();

        layers
            .iter_mut()
            .map(|layer| layer.draw(state, graphics_state))
            .fold(Ok(&mut builder), |res, cmd| {
                res.and_then(|builder| builder.execute_commands(cmd))
            })
            .unwrap();

        builder
            .next_subpass(SubpassContents::SecondaryCommandBuffers)
            .unwrap();

        builder
            .execute_commands(
                state
                    .start_mutate()
                    .get_mut(|gui: &mut GuiState| {
                        gui.init_draw_on_subpass_image(target_image_size.clone())
                    })
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
