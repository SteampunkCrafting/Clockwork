use crate::event::Event;
use crate::state::*;
use clockwork_core::prelude::*;
use egui_winit_vulkano::Gui;
use log::*;
use std::*;
use winit::{
    event::{Event as WinitEvent, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

/// A winit-based main loop
pub fn main_loop<S>(mut state: S, mut mechanisms: Mechanisms<S, Event>)
where
    S: CallbackSubstate<IOState> + CallbackSubstate<MainLoopState> + CallbackSubstate<Option<Gui>>,
{
    /* ---- INITIALIZATION ---- */
    info!("Initializing main loop");
    let event_loop = EventLoop::<Event>::with_user_event();
    let event_proxy = event_loop.create_proxy();

    /* -- ADDING EVENT LOOP OBJECT TO THE STATE -- */
    info!("Adding winit event loop to the engine state");
    state.callback_substate_mut(|MainLoopState(el)| *el = Some(event_loop));
    info!("Done adding winit event loop to the engine state");

    /* -- INITIALIZING MECHANISMS -- */
    info!("Finished initialization of the main loop");
    info!("Initializing mechanisms");
    mechanisms.clink_event(&mut state, Event::Initialization);
    info!("Finished initializing mechanisms");

    /* -- TAKING BACK EVENT LOOP OBJECT FROM THE STATE -- */
    info!("Retrieving event loop object from the engine state");
    let event_loop = {
        let mut event_loop = None;
        state.callback_substate_mut(|MainLoopState(el)| event_loop = el.take());
        event_loop.unwrap()
    };
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

        /* ---- UPDATING GUI, IF EXISTS ---- */
        CallbackSubstate::<Option<Gui>>::callback_substate_mut(&mut state, |gui| {
            gui.as_mut()
                .expect("Fatal: GUI has not been initialized")
                .update(&ev);
        });

        /* ---- HANDLING EVENT ---- */
        match ev {
            WinitEvent::UserEvent(Event::Tick(delta_time)) => {
                debug!("Performing tick (delta time: {:?})", delta_time);
                mechanisms.clink_event(&mut state, Event::Tick(delta_time));
                debug!("Finished tick");
            }
            WinitEvent::UserEvent(Event::Draw(delta_time)) => {
                debug!("Performing draw call (delta time: {:?})", delta_time);
                mechanisms.clink_event(&mut state, Event::Draw(delta_time));
                debug!("Finished draw call");
            }
            WinitEvent::LoopDestroyed => {
                info!("Terminating main loop");
                info!("Terminating mechanisms");
                mechanisms.clink_event(&mut state, Event::Termination);
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
                state.callback_substate_mut(
                    |IOState {
                         input:
                             Input {
                                 pressed_keys: pk, ..
                             },
                         ..
                     }| match keyboard_state {
                        winit::event::ElementState::Pressed => {
                            pk.insert(vkk);
                        }
                        winit::event::ElementState::Released => {
                            pk.remove(&vkk);
                        }
                    },
                );
            }
            WinitEvent::MainEventsCleared => {
                let (desired_tick_period, desired_min_draw_period) = {
                    let mut res = None;
                    state.callback_substate_mut(
                        |IOState {
                             desired_tick_period,
                             desired_min_draw_period,
                             ..
                         }| {
                            res = Some((
                                desired_tick_period.clone(),
                                desired_min_draw_period.clone(),
                            ));
                        },
                    );
                    res.unwrap()
                };
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
                            .send_event(Event::Tick(tick_delta))
                            .map_or((), |_| ())
                    }
                    (_, draw_delta) if draw_debt => {
                        est_draw_period = draw_delta;
                        frames_total += 1;
                        draw_debt = false;
                        last_draw_start_at = time::Instant::now();
                        event_proxy
                            .send_event(Event::Draw(draw_delta))
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
                state.callback_substate_mut(
                    |IOState {
                         statistics:
                             Statistics {
                                 ticks_total: state_ticks_total,
                                 frames_total: state_frames_total,
                                 tick_period: state_tick_period,
                                 draw_period: state_draw_period,
                                 ..
                             },
                         ..
                     }| {
                        *state_ticks_total = ticks_total;
                        *state_frames_total = frames_total;
                        *state_tick_period = est_tick_period;
                        *state_draw_period = est_draw_period;
                    },
                );
            }
            _ => {}
        };
        trace!("Finished handling event: {:?}", ev);
    });
}
