use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub struct Lock<T>(RwLock<T>);
impl<'a, T> Lock<T> {
    pub fn lock(&'a self) -> ReadGuard<'a, T> {
        ReadGuard(self.0.read().expect("Poisoning Error"))
    }

    pub fn lock_mut(&'a mut self) -> WriteGuard<'a, T> {
        WriteGuard(self.0.write().expect("Poisoning Error"))
    }
}
impl<T> From<T> for Lock<T> {
    fn from(x: T) -> Self {
        Lock(RwLock::new(x))
    }
}

pub struct ReadGuard<'a, T>(RwLockReadGuard<'a, T>);
impl<T> Borrow<T> for ReadGuard<'_, T> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}
impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.borrow()
    }
}

pub struct WriteGuard<'a, T>(RwLockWriteGuard<'a, T>);
impl<T> Borrow<T> for WriteGuard<'_, T> {
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}
impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.borrow()
    }
}
impl<T> BorrowMut<T> for WriteGuard<'_, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.0.borrow_mut()
    }
}
impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.borrow_mut()
    }
}
