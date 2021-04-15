pub mod clockwork;
pub mod mechanism;
pub mod sync {
    use std::{
        borrow::{Borrow, BorrowMut},
        ops::{Deref, DerefMut},
        sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    };

    pub struct Lock<T>(RwLock<T>);
    impl<'a, T> Lock<T> {
        pub fn lock(&'a self) -> LockReadGuard<'a, T> {
            LockReadGuard(self.0.read().expect("Poisoning Error"))
        }

        pub fn lock_mut(&'a mut self) -> LockWriteGuard<'a, T> {
            LockWriteGuard(self.0.write().expect("Poisoning Error"))
        }
    }
    impl<T> From<T> for Lock<T> {
        fn from(x: T) -> Self {
            Lock(RwLock::new(x))
        }
    }

    pub struct LockReadGuard<'a, T>(RwLockReadGuard<'a, T>);
    impl<T> Deref for LockReadGuard<'_, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            self.0.borrow()
        }
    }

    pub struct LockWriteGuard<'a, T>(RwLockWriteGuard<'a, T>);
    impl<T> Deref for LockWriteGuard<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.0.borrow()
        }
    }
    impl<T> DerefMut for LockWriteGuard<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0.borrow_mut()
        }
    }
}

pub mod prelude {
    pub use crate::clockwork::*;
    pub use crate::mechanism::*;
    pub use crate::sync::*;
}
