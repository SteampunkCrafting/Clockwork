use clockwork_core::sync::{ReadLock, WriteLock};
use spc_clockwork_asset_storage::prelude::*;
use std::thread;

#[derive(EnumIter, Clone, Copy, Hash, Eq, PartialEq)]
enum StorageKey {
    A,
    B,
    C,
}

static MAPPING: fn(StorageKey) -> i32 = |x| match x {
    StorageKey::A => 0,
    StorageKey::B => 1,
    StorageKey::C => 2,
};

#[test]
fn synchronous_access() {
    let access_count: WriteLock<u8> = 0.into();

    let ac = access_count.clone();
    let storage: AssetStorage<StorageKey, i32> = (move |x| {
        *ac.lock_mut() += 1;
        MAPPING(x)
    })
    .into();

    (0..100).for_each(|_| {
        assert_eq!(*storage.get(StorageKey::A).lock(), 0);
        assert_eq!(*storage.get(StorageKey::B).lock(), 1);
        assert_eq!(*storage.get(StorageKey::C).lock(), 2);
        assert_eq!(*access_count.lock(), 3);
    })
}

#[test]
fn parallel_access() {
    let access_count: WriteLock<u8> = 0.into();

    let ac = access_count.clone();
    let storage = ReadLock::from(AssetStorage::<StorageKey, i32>::from(move |x| {
        *ac.lock_mut() += 1;
        MAPPING(x)
    }));

    (0..10)
        .map(|_| {
            let storage = storage.clone();
            let access_count = access_count.clone();
            thread::spawn(move || {
                (0..100).for_each(|_| {
                    let storage = storage.lock();
                    assert_eq!(*storage.get(StorageKey::A).lock(), 0);
                    assert_eq!(*storage.get(StorageKey::B).lock(), 1);
                    assert_eq!(*storage.get(StorageKey::C).lock(), 2);
                    assert_eq!(*access_count.lock(), 3);
                })
            })
        })
        .for_each(|thread| thread.join().unwrap())
}
