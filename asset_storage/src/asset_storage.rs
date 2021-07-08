use clockwork_core::sync::ReadLock;
use std::{cell::RefCell, collections::HashMap, hash::Hash};
use strum::IntoEnumIterator;

pub use strum_macros::EnumIter;

pub trait AssetStorageKey: Clone + Hash + Eq + IntoEnumIterator {}

pub struct AssetStorage<T, U>(Box<dyn Send + Fn(T) -> U>, RefCell<HashMap<T, ReadLock<U>>>)
where
    T: AssetStorageKey;

impl<T> AssetStorageKey for T where T: Clone + Hash + Eq + IntoEnumIterator {}

impl<T, U> AssetStorage<T, U>
where
    T: AssetStorageKey,
{
    pub fn get(&self, key: T) -> ReadLock<U> {
        let Self(eval, map) = self;

        if let Some(asset) = map.borrow().get(&key) {
            return asset.clone();
        }

        let asset = ReadLock::from(eval(key.clone()));
        map.borrow_mut().insert(key, asset.clone());
        asset
    }

    pub fn eval_all(&self) {
        let _ = T::iter().map(|key| self.get(key)).collect::<Vec<_>>();
    }
}

impl<F, T, U> From<F> for AssetStorage<T, U>
where
    F: Send + Fn(T) -> U + 'static,
    T: AssetStorageKey,
{
    fn from(eval_fn: F) -> Self {
        Self(Box::new(eval_fn), Default::default())
    }
}
