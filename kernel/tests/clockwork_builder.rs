use spc_clockwork_kernel::prelude::*;
use std::*;

#[test]
fn successful_construction() {
    assert!(Clockwork::<TestState, TestEvent>::builder()
        .with_state(TestState::default())
        .with_main_loop(|_, _| {})
        .build()
        .is_ok())
}

#[test]
fn missing_initial_state() {
    assert!(Clockwork::<TestState, TestEvent>::builder()
        .with_main_loop(|_, _| {})
        .build()
        .map_err(|msg| assert_eq!(msg, "Missing initial state"))
        .is_err());
}

#[test]
fn missing_main_loop() {
    assert!(Clockwork::<TestState, TestEvent>::builder()
        .with_state(TestState::default())
        .build()
        .map_err(|msg| assert_eq!(msg, "Missing main loop"))
        .is_err());
}

type TestState = i32;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct TestEvent;
impl fmt::Display for TestEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
