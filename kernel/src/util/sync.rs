#[cfg(debug_assertions)]
use no_deadlocks::{RwLock, RwLockReadGuard, RwLockWriteGuard};
#[cfg(not(debug_assertions))]
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::{
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed},
        Arc,
    },
};

static GLOBAL_LOCK_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct WriteLock<T>(usize, Arc<RwLock<T>>);

pub struct ReadLock<T>(WriteLock<T>);

pub struct Lock<T>(WriteLock<T>);

unsafe impl<T> Send for ReadLock<T> {}
unsafe impl<T> Sync for ReadLock<T> {}
unsafe impl<T> Send for WriteLock<T> {}
unsafe impl<T> Sync for WriteLock<T> {}
unsafe impl<T> Send for Lock<T> {}
unsafe impl<T> Sync for Lock<T> {}

pub struct Guard<'a, T>(RwLockWriteGuard<'a, T>);

pub struct ReadGuard<'a, T>(RwLockReadGuard<'a, T>);

impl<T> From<T> for WriteLock<T> {
    fn from(x: T) -> Self {
        Self(
            GLOBAL_LOCK_COUNTER.fetch_add(1, Relaxed),
            Arc::new(RwLock::new(x)),
        )
    }
}

impl<T> Default for WriteLock<T>
where
    T: Default,
{
    fn default() -> Self {
        T::default().into()
    }
}

impl<T> Clone for WriteLock<T> {
    fn clone(&self) -> Self {
        let Self(id, lk) = self;
        Self(id.clone(), lk.clone())
    }
}

impl<T> WriteLock<T> {
    pub fn id(&self) -> usize {
        let Self(id, _) = self;
        id.clone()
    }

    pub fn downgrade_to_read_lock(&self) -> ReadLock<T> {
        ReadLock(self.clone())
    }

    pub fn downgrade_to_user_lock(&self) -> Lock<T> {
        Lock(self.clone())
    }

    pub fn lock(&self) -> ReadGuard<'_, T> {
        let Self(_, rw_lock) = self;
        ReadGuard(rw_lock.read().unwrap())
    }

    pub fn lock_mut(&self) -> Guard<'_, T> {
        let Self(_, rw_lock) = self;
        Guard(rw_lock.write().unwrap())
    }
}

impl<T> From<T> for ReadLock<T> {
    fn from(x: T) -> Self {
        ReadLock(x.into())
    }
}

impl<T> Default for ReadLock<T>
where
    T: Default,
{
    fn default() -> Self {
        T::default().into()
    }
}

impl<T> Clone for ReadLock<T> {
    fn clone(&self) -> Self {
        let Self(inner) = self;
        Self(inner.clone())
    }
}

impl<T> ReadLock<T> {
    pub fn id(&self) -> usize {
        let Self(inner) = self;
        inner.id()
    }

    pub fn lock(&self) -> ReadGuard<'_, T> {
        let Self(inner) = self;
        inner.lock()
    }
}

impl<T> From<T> for Lock<T> {
    fn from(x: T) -> Self {
        Self(WriteLock::from(x))
    }
}

impl<T> Default for Lock<T>
where
    T: Default,
{
    fn default() -> Self {
        T::default().into()
    }
}

impl<T> Lock<T> {
    pub fn id(&self) -> usize {
        let Self(inner) = self;
        inner.id()
    }

    pub fn lock(&self) -> ReadGuard<'_, T> {
        let Self(inner) = self;
        inner.lock()
    }

    pub fn lock_mut(&mut self) -> Guard<'_, T> {
        let Self(inner) = self;
        inner.lock_mut()
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
