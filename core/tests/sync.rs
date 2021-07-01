use spc_clockwork_core::sync::*;
use std::thread;

#[test]
fn parallel_counter_increment() {
    let counter = Lock::from(0u16);
    (0..5)
        .into_iter()
        .map(|_| counter.clone())
        .map(|count| {
            thread::spawn(move || (0..500).into_iter().for_each(|_| *count.lock_mut() += 1))
        })
        .for_each(|t| t.join().unwrap());
    assert_eq!(*counter.downgrade().lock(), 2500)
}
