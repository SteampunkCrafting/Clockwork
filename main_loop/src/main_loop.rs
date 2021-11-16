use crate::state::WinitLoopState;
use crate::state::{InputState, MainLoopStatistics};
use kernel::abstract_runtime::{CallbackSubstate, EngineState, Mechanisms};
use kernel::prelude::*;
use kernel::standard_runtime::FromIntoStandardEvent;
use std::*;
use winit::{
    event::{Event as WinitEvent, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
};

pub fn main_loop<S, E>(mut state: EngineState<S>, mut mechanisms: Mechanisms<S, E>)
where
    S: CallbackSubstate<MainLoopStatistics>
        + CallbackSubstate<WinitLoopState<E>>
        + CallbackSubstate<InputState>,
    E: FromIntoStandardEvent,
{
    /* -- INITIALIZING MECHANISMS -- */
    info!("Finished initialization of the main loop");
    info!("Initializing mechanisms");
    mechanisms.clink_event(&mut state, StandardEvent::Initialization.into());
    info!("Finished initializing mechanisms");

    /* -- TAKING BACK EVENT LOOP OBJECT FROM THE STATE -- */
    info!("Retrieving event loop object from the engine state");
    let (event_loop, event_proxy) = state
        .start_mutate()
        .get_mut(|s: &mut WinitLoopState<E>| s.initialize())
        .finish();
    info!("Done retrieving event loop object from the engine state");

    /* ---- EVENT LOOP LAUNCH ---- */
    info!("Starting main loop");

    let mut last_tick_start_at = time::Instant::now();
    let mut last_draw_start_at = time::Instant::now();
    let mut tick_debt = 0f32;
    let mut draw_debt = false;
    let mut ticks_total = 0;
    let mut frames_total = 0;
    event_loop.run(move |ev, _, cf| {
        trace!("Handling next event: {:?}", ev);
        let current_time = time::Instant::now();

        /* ---- NOTIFYING SUBSCRIBERS ABOUT THE EVENT ---- */
        state
            .start_mutate()
            .get_mut(|ml: &mut WinitLoopState<E>| ml.notify(&ev))
            .finish();

        /* ---- HANDLING EVENT ---- */
        match ev {
            WinitEvent::UserEvent(ref ev) if ev.clone().into() == StandardEvent::Tick => {
                debug!("Performing tick");
                mechanisms.clink_event(&mut state, StandardEvent::Tick.into());
                debug!("Finished tick");
            }
            WinitEvent::UserEvent(ref ev) if ev.clone().into() == StandardEvent::Draw => {
                debug!("Performing draw call");
                mechanisms.clink_event(&mut state, StandardEvent::Draw.into());
                debug!("Finished draw call");
            }
            WinitEvent::LoopDestroyed => {
                info!("Terminating main loop");
                info!("Terminating mechanisms");
                mechanisms.clink_event(&mut state, StandardEvent::Termination.into());
                info!("Finished terminating mechanisms");
            }
            WinitEvent::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *cf = ControlFlow::Exit,
            WinitEvent::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: keyboard_state,
                                virtual_keycode: Some(vkk),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                state
                    .start_mutate()
                    .get_mut(|InputState { pressed_keys, .. }| match keyboard_state {
                        winit::event::ElementState::Pressed => pressed_keys.insert(vkk),
                        winit::event::ElementState::Released => pressed_keys.remove(&vkk),
                    })
                    .finish();
            }
            WinitEvent::MainEventsCleared => {
                let (desired_tick_period, desired_min_draw_period) = state
                    .start_mutate()
                    .get(
                        |MainLoopStatistics {
                             desired_avg_tick_period,
                             desired_min_draw_period,
                             ..
                         }| {
                            (*desired_avg_tick_period, *desired_min_draw_period)
                        },
                    )
                    .finish();
                let mut est_tick_period = Default::default();
                let mut est_draw_period = Default::default();

                match (
                    current_time - last_tick_start_at,
                    current_time - last_draw_start_at,
                ) {
                    (tick_delta, _) if tick_debt >= 1f32 => {
                        est_tick_period = tick_delta;
                        ticks_total += 1;
                        last_tick_start_at = time::Instant::now();
                        tick_debt -= 1f32;
                        event_proxy
                            .send_event(StandardEvent::Tick.into())
                            .map_or((), |_| ())
                    }
                    (_, draw_delta) if draw_debt => {
                        est_draw_period = draw_delta;
                        frames_total += 1;
                        draw_debt = false;
                        last_draw_start_at = time::Instant::now();
                        event_proxy
                            .send_event(StandardEvent::Draw.into())
                            .map_or((), |_| ());
                    }
                    (tick_delta, draw_delta) => {
                        tick_debt = tick_delta.as_secs_f32() / desired_tick_period.as_secs_f32();
                        draw_debt = draw_delta.as_secs_f32()
                            / desired_min_draw_period.as_secs_f32()
                            >= 1f32;
                        *cf = if tick_debt >= 1f32 || draw_debt {
                            ControlFlow::Poll
                        } else {
                            ControlFlow::WaitUntil(cmp::min(
                                last_tick_start_at + desired_tick_period,
                                last_draw_start_at + desired_min_draw_period,
                            ))
                        }
                    }
                }
                state
                    .start_mutate()
                    .get_mut(
                        |MainLoopStatistics {
                             current_tick_delta: state_tick_period,
                             current_draw_delta: state_draw_period,
                             ticks_total: state_ticks_total,
                             frames_total: state_frames_total,
                             ..
                         }| {
                            *state_ticks_total = ticks_total;
                            *state_frames_total = frames_total;
                            *state_tick_period = est_tick_period;
                            *state_draw_period = est_draw_period;
                        },
                    )
                    .finish()
            }
            _ => {}
        };
        trace!("Finished handling event: {:?}", ev);
    });
}
