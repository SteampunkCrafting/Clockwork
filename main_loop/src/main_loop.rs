use crate::state::InitWinitState;
use crate::state::{InputState, MainLoopStatistics};
use kernel::abstract_runtime::{EngineState, Mechanisms, Substate};
use kernel::prelude::*;
use kernel::standard_runtime::StandardEventSuperset;
use std::convert::TryInto;
use std::*;
use winit::{
    event::{Event as WinitEvent, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
};

pub fn main_loop<S, E>(mut state: EngineState<S>, mut mechanisms: Mechanisms<S, E>)
where
    S: Substate<MainLoopStatistics> + Substate<InitWinitState<E>> + Substate<InputState>,
    E: StandardEventSuperset,
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
        .get_mut(|s: &mut InitWinitState<E>| s.initialize())
        .finish();
    info!("Done retrieving event loop object from the engine state");

    /* ---- EVENT LOOP LAUNCH ---- */
    info!("Starting main loop");

    let mut last_tick_start_at = time::Instant::now();
    let mut last_draw_start_at = time::Instant::now();
    let mut est_tick_period = Default::default();
    let mut est_draw_period = Default::default();
    let mut tick_debt = 0f32;
    let mut draw_debt = 0f32;
    let mut ticks_total = 0;
    let mut frames_total = 0;

    event_loop.run(move |ev, _, cf| {
        trace!("Handling next event: {:?}", ev);
        let current_time = time::Instant::now();

        /* ---- NOTIFYING SUBSCRIBERS ABOUT THE EVENT ---- */
        state
            .start_mutate()
            .get_mut(|ml: &mut InitWinitState<E>| ml.notify(&ev))
            .finish();

        /* ---- HANDLING EVENT ---- */
        match ev {
            WinitEvent::UserEvent(ref event)
                if TryInto::<StandardEvent>::try_into(event.clone())
                    .map(|standard_event| standard_event == StandardEvent::Termination)
                    .map_err(|_| ())
                    .and_then(|x| x.then(|| ()).ok_or(()))
                    .is_ok() =>
            {
                debug!("Requested main loop termination");
                debug!("Terminating mechanisms");
                mechanisms.clink_event(&mut state, event.clone());
                debug!("Terminating main loop");
                *cf = ControlFlow::Exit;
                debug!("Finished main loop termination");
            }
            WinitEvent::UserEvent(ref event) => {
                if let Ok(standard_event) = TryInto::<StandardEvent>::try_into(event.clone()) {
                    debug!("Handling standard event: {:?}", &standard_event);
                    mechanisms.clink_event(&mut state, event.clone());
                    debug!("Finished handling standard event: {:?}", &standard_event);
                } else {
                    debug!("Handling custom event: {:?}", &event);
                    mechanisms.clink_event(&mut state, event.clone());
                    debug!("Finished handling custom event: {:?}", &event);
                }
            }
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
                    (_, draw_delta) if draw_debt >= 1f32 => {
                        est_draw_period = draw_delta;
                        frames_total += 1;
                        draw_debt = 0f32;
                        last_draw_start_at = time::Instant::now();
                        event_proxy
                            .send_event(StandardEvent::Draw.into())
                            .map_or((), |_| ())
                    }
                    (tick_delta, draw_delta) => {
                        tick_debt += tick_delta.as_secs_f32() / desired_tick_period.as_secs_f32();
                        draw_debt +=
                            draw_delta.as_secs_f32() / desired_min_draw_period.as_secs_f32();
                        *cf = if tick_debt >= 1f32 || draw_debt >= 1f32 {
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
