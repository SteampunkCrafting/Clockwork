use spc_clockwork_kernel::util::sync::WriteLock;
use std::thread;

#[test]
fn parallel_counter_increment() {
    let counter = WriteLock::from(0u16);
    (0..5)
        .into_iter()
        .map(|_| counter.clone())
        .map(|count| {
            thread::spawn(move || (0..500).into_iter().for_each(|_| *count.lock_mut() += 1))
        })
        .for_each(|t| t.join().unwrap());
    assert_eq!(*counter.downgrade_to_read_lock().lock(), 2500)
}
