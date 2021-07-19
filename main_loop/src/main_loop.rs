use crate::event::Event;
use crate::state::*;
use clockwork_core::prelude::*;
use log::*;
use std::*;
use winit::{
    event::Event::{LoopDestroyed, MainEventsCleared, UserEvent},
    event_loop::{ControlFlow, EventLoop},
};

/// A winit-based main loop
pub fn main_loop<S>(mut state: S, mut mechanisms: Mechanisms<S, Event>)
where
    S: Substate<IOState>,
{
    /* ---- INITIALIZATION ---- */
    info!("Initializing main loop");
    let event_loop = EventLoop::<Event>::with_user_event();
    let event_proxy = event_loop.create_proxy();

    /* -- ADDING EVENT LOOP OBJECT TO THE STATE -- */
    info!("Adding winit event loop to the engine state");
    Substate::<IOState>::substate_mut(&mut state).event_loop = Some(event_loop);
    info!("Done adding winit event loop to the engine state");

    /* -- INITIALIZING MECHANISMS -- */
    info!("Finished initialization of the main loop");
    info!("Initializing mechanisms");
    mechanisms.clink_event(&mut state, Event::Initialization);
    info!("Finished initializing mechanisms");

    /* -- TAKING BACK EVENT LOOP OBJECT FROM THE STATE -- */
    info!("Retreiving event loop object from the engine state");
    let event_loop = Substate::<IOState>::substate_mut(&mut state)
        .event_loop
        .take()
        .expect("Winit event loop has been taken by a mechanism during the initialization process");
    info!("Done retreiving event loop object from the engine state");

    /* ---- EVENT LOOP LAUNCH ---- */
    info!("Starting main loop");

    let mut last_tick_start_at = time::Instant::now();
    let mut last_draw_start_at = time::Instant::now();
    let mut tick_debt = 0f32;
    let mut draw_debt = false;

    event_loop.run(move |ev, _, cf| {
        trace!("Handling next event: {:?}", ev);
        let current_time = time::Instant::now();
        let IOState {
            desired_tick_period,
            desired_min_draw_period,
            statistics:
                Statistics {
                    draw_period: est_draw_period,
                    tick_period: est_tick_period,
                    ticks_total,
                    frames_total,
                    ..
                },
            ..
        } = Substate::<IOState>::substate_mut(&mut state);
        match ev {
            UserEvent(Event::Tick(delta_time)) => {
                debug!("Performing tick (delta time: {:?})", delta_time);
                mechanisms.clink_event(&mut state, Event::Tick(delta_time));
                debug!("Finished tick");
            }
            UserEvent(Event::Draw(delta_time)) => {
                debug!("Performing draw call (delta time: {:?})", delta_time);
                mechanisms.clink_event(&mut state, Event::Draw(delta_time));
                debug!("Finished draw call");
            }
            LoopDestroyed => {
                info!("Terminating main loop");
                info!("Terminating mechanisms");
                mechanisms.clink_event(&mut state, Event::Termination);
                info!("Finished terminating mechanisms");
            }
            MainEventsCleared => match (
                current_time - last_tick_start_at,
                current_time - last_draw_start_at,
            ) {
                (tick_delta, _) if tick_debt >= 1f32 => {
                    *est_tick_period = tick_delta;
                    *ticks_total += 1;
                    last_tick_start_at = time::Instant::now();
                    tick_debt -= 1f32;
                    event_proxy
                        .send_event(Event::Tick(tick_delta))
                        .map_or((), |_| ())
                }
                (_, draw_delta) if draw_debt => {
                    *est_draw_period = draw_delta;
                    *frames_total += 1;
                    draw_debt = false;
                    last_draw_start_at = time::Instant::now();
                    event_proxy
                        .send_event(Event::Draw(draw_delta))
                        .map_or((), |_| ());
                }
                (tick_delta, draw_delta) => {
                    tick_debt = tick_delta.as_secs_f32() / desired_tick_period.as_secs_f32();
                    draw_debt =
                        draw_delta.as_secs_f32() / desired_min_draw_period.as_secs_f32() >= 1f32;
                    *cf = if tick_debt >= 1f32 || draw_debt {
                        ControlFlow::Poll
                    } else {
                        ControlFlow::WaitUntil(cmp::min(
                            last_tick_start_at + *desired_tick_period,
                            last_draw_start_at + *desired_min_draw_period,
                        ))
                    }
                }
            },
            _ => {}
        };
        trace!("Finished handling event: {:?}", ev);
    });
}
