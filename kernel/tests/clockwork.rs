use spc_clockwork_kernel::prelude::*;
use std::*;

#[test]
fn clockwork_execution() {
    Clockwork::<TestState, TestEvent>::builder()
        .with_state(0)
        .with_main_loop(test_main_loop)
        .with_mechanism(TestMechanism(1))
        .with_mechanism(TestMechanism(2))
        .with_mechanism(TestMechanism(3))
        .with_mechanism(TestMechanism(0))
        .with_read_mechanism(TestReadMechanism(0, 6))
        .with_read_mechanism(TestReadMechanism(0, 6))
        .build()
        .unwrap()
        .set_the_clock();

    fn test_main_loop(mut state: TestState, mut mechanisms: Mechanisms<TestState, TestEvent>) {
        use borrow::BorrowMut;

        let (state, mechanisms) = (state.borrow_mut(), mechanisms.borrow_mut());
        mechanisms.clink_event(state, TestEvent::Start);
        while *state < 10 {
            mechanisms.clink_event(state, TestEvent::Tick);
        }
        mechanisms.clink_event(state, TestEvent::Termination);
    }

    struct TestMechanism(i32);
    impl Mechanism<TestState, TestEvent> for TestMechanism {
        fn name(&self) -> &'static str {
            "Test Mechanism"
        }

        fn clink(&mut self, state: &mut i32, event: TestEvent) {
            let Self(inc) = self;
            match event {
                TestEvent::Tick => *state += *inc,
                _ => panic!("Mechanism did not subscribe to this event type"),
            }
        }

        fn handled_events(&self) -> Option<&'static [TestEvent]> {
            Some(&[TestEvent::Tick])
        }
    }

    struct TestReadMechanism(pub i32, pub i32);
    impl ReadMechanism<TestState, TestEvent> for TestReadMechanism {
        fn name(&self) -> &'static str {
            "Test Read Mechanism"
        }

        fn clink(&mut self, state: &i32, event: TestEvent) {
            let Self(prev_state, expect_inc) = self;
            match event {
                TestEvent::Tick => assert_eq!(*prev_state + *expect_inc, *state),
                TestEvent::Termination => assert_eq!(*state, 12),
                _ => panic!("Mechanism did not subscribe to this event type"),
            }
            *prev_state += *expect_inc;
        }

        fn handled_events(&self) -> Option<&'static [TestEvent]> {
            Some(&[TestEvent::Tick, TestEvent::Termination])
        }
    }

    type TestState = i32;

    #[derive(Clone, Hash, Eq, PartialEq, Debug)]
    enum TestEvent {
        Start,
        Tick,
        Termination,
    }
    impl fmt::Display for TestEvent {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
}
