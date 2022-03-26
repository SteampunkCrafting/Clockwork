use ambassador::delegatable_trait;

/// A set of constraints, which every valid Clockwork state should satisfy.
///
/// This trait is implemented automatically on every type, which
/// is theoretically possible to use as a state.
///
/// Although, it is still required to manually mark the data structure
/// as a Clockwork state via the `ClockworkState` trait.
pub trait ClockworkStateRequirements
where
    Self: Sized + 'static,
{
}
impl<T> ClockworkStateRequirements for T where T: Sized + 'static {}

/// A marker trait for the clockwork state.
///
/// Must be implemented on every substate of a clockwork state,
/// as well as on the superstate.
pub trait ClockworkState
where
    Self: ClockworkStateRequirements,
{
}

/// A substate of a clockwork state, accessible via callback.
///
/// This is a more general type trait, which should be requested by
/// mechanisms.
#[delegatable_trait]
pub trait Substate<S>
where
    Self: ClockworkState,
    S: ClockworkState + ?Sized,
{
    /// Executes provided callback, supplying its substate reference
    fn substate<R>(&self, callback: impl FnOnce(&S) -> R) -> R;
    /// Executes provided callback, supplying its mutable substate reference
    fn substate_mut<R>(&mut self, callback: impl FnOnce(&mut S) -> R) -> R;
}

/// Any ClockworkState is a substate of itself.
///
/// > Very strange from a logical point of view, though very convenient
impl<T> Substate<T> for T
where
    T: ClockworkState,
{
    fn substate<R>(&self, callback: impl FnOnce(&T) -> R) -> R {
        callback(self)
    }

    fn substate_mut<R>(&mut self, callback: impl FnOnce(&mut T) -> R) -> R {
        callback(self)
    }
}

/// A wrapper struct for the engine state that
/// allows to access the substate objects through
/// callback guards.
pub struct EngineState<S>(pub(crate) S)
where
    S: ClockworkState;

/// A callback guard, which enables Clockwork State reading.
pub struct ReadCallbackGuard<'a, S, R = ()>
where
    S: ClockworkState,
{
    state: &'a EngineState<S>,
    result: R,
}

/// A callback guard, which enables Clockwork State writing.
pub struct WriteCallbackGuard<'a, S, R = ()>
where
    S: ClockworkState,
{
    state: &'a mut EngineState<S>,
    result: R,
}

impl<S> EngineState<S>
where
    S: ClockworkState,
{
    /// Constructs a read callback guard,
    /// allowing read-only access to its substate objects through a callback.
    pub fn start_access(&self) -> ReadCallbackGuard<'_, S, ()> {
        ReadCallbackGuard {
            state: self,
            result: (),
        }
    }

    /// Constructs a write callback guard,
    /// allowing read-write access to the substate objects
    /// of this clockwork state, then executes a callback on it.
    pub fn start_mutate(&mut self) -> WriteCallbackGuard<'_, S, ()> {
        WriteCallbackGuard {
            result: (),
            state: self,
        }
    }
}

impl<'a, S> ReadCallbackGuard<'a, S, ()>
where
    S: ClockworkState,
{
    /// Executes the callback, which takes a reference to a substate,
    /// returning some result.
    ///
    /// The method is only available for ReadCallbackGuards with empty return result.
    pub fn get<T, U>(self, callback: impl FnOnce(&T) -> U) -> ReadCallbackGuard<'a, S, U>
    where
        T: ClockworkState,
        S: Substate<T>,
    {
        let Self { state, .. } = self;
        ReadCallbackGuard {
            result: state.0.substate(|state| callback(state)),
            state,
        }
    }
}

impl<'a, S, R> ReadCallbackGuard<'a, S, R>
where
    S: ClockworkState,
{
    /// Destroys the callback guard, returning accumulated result
    pub fn finish(self) -> R {
        self.result
    }

    /// Executes the callback, which takes a reference to a substate,
    /// and the accumulated result, returning another accumulated result.
    pub fn then_get<T, U>(self, callback: impl FnOnce(R, &T) -> U) -> ReadCallbackGuard<'a, S, U>
    where
        T: ClockworkState,
        S: Substate<T>,
    {
        let Self { state, result } = self;
        ReadCallbackGuard {
            result: state.0.substate(move |state| callback(result, state)),
            state,
        }
    }

    /// Executes a callback, which take a reference to a substate,
    /// and then zips its return to the existing result.
    pub fn then_get_zip<T, U>(
        self,
        callback: impl FnOnce(&T) -> U,
    ) -> ReadCallbackGuard<'a, S, (R, U)>
    where
        T: ClockworkState,
        S: Substate<T>,
    {
        let Self { state, result } = self;
        ReadCallbackGuard {
            result: (result, state.0.substate(callback)),
            state,
        }
    }

    /// Executes a callback on a result without performing a reading from a substate.
    pub fn map<U>(self, callback: impl FnOnce(R) -> U) -> ReadCallbackGuard<'a, S, U> {
        let Self { state, result } = self;
        ReadCallbackGuard {
            result: callback(result),
            state,
        }
    }
}

impl<'a, S> WriteCallbackGuard<'a, S, ()>
where
    S: ClockworkState,
{
    /// Executes the callback, which takes a reference to a substate,
    /// returning some result.
    ///
    /// The method is only available for WriteCallbackGuards with empty return result.
    pub fn get<T, U>(self, callback: impl FnOnce(&T) -> U) -> WriteCallbackGuard<'a, S, U>
    where
        T: ClockworkState,
        S: Substate<T>,
    {
        let Self { state, .. } = self;
        WriteCallbackGuard {
            result: state.0.substate(|state| callback(state)),
            state,
        }
    }

    /// Executes the callback, which takes a reference to a substate,
    /// returning some result.
    ///
    /// The method is only available for WriteCallbackGuards with empty return result.
    pub fn get_mut<T, U>(self, callback: impl FnOnce(&mut T) -> U) -> WriteCallbackGuard<'a, S, U>
    where
        T: ClockworkState,
        S: Substate<T>,
    {
        let Self { state, .. } = self;
        WriteCallbackGuard {
            result: state.0.substate_mut(|state| callback(state)),
            state,
        }
    }
}

impl<'a, S, R> WriteCallbackGuard<'a, S, R>
where
    S: ClockworkState,
{
    /// Destroys the callback guard, returning accumulated result
    pub fn finish(self) -> R {
        self.result
    }

    /// Executes the callback, which takes a reference to a substate,
    /// and the accumulated result, returning another accumulated result.
    pub fn then_get<T, U>(self, callback: impl FnOnce(R, &T) -> U) -> WriteCallbackGuard<'a, S, U>
    where
        T: ClockworkState,
        S: Substate<T>,
    {
        let Self { state, result } = self;
        WriteCallbackGuard {
            result: state.0.substate(move |state| callback(result, state)),
            state,
        }
    }

    /// Executes a callback, which take a reference to a substate,
    /// and then zips its return to the existing result.
    pub fn then_get_zip<T, U>(
        self,
        callback: impl FnOnce(&T) -> U,
    ) -> WriteCallbackGuard<'a, S, (R, U)>
    where
        T: ClockworkState,
        S: Substate<T>,
    {
        let Self { state, result } = self;
        WriteCallbackGuard {
            result: (result, state.0.substate(callback)),
            state,
        }
    }

    /// Executes the callback, which takes a mutable reference to a substate,
    /// and the accumulated result, returning another accumulated result.
    pub fn then_get_mut<T, U>(
        self,
        callback: impl FnOnce(R, &mut T) -> U,
    ) -> WriteCallbackGuard<'a, S, U>
    where
        T: ClockworkState,
        S: Substate<T>,
    {
        let Self { state, result } = self;
        WriteCallbackGuard {
            result: state.0.substate_mut(move |state| callback(result, state)),
            state,
        }
    }

    /// Executes a callback, which take a mutable reference to a substate,
    /// and then zips its return to the existing result.
    pub fn then_get_zip_mut<T, U>(
        self,
        callback: impl FnOnce(&mut T) -> U,
    ) -> WriteCallbackGuard<'a, S, (R, U)>
    where
        T: ClockworkState,
        S: Substate<T>,
    {
        let Self { state, result } = self;
        WriteCallbackGuard {
            result: (result, state.0.substate_mut(callback)),
            state,
        }
    }

    /// Executes a callback on a result without performing any reading from a substate, or writing to it.
    pub fn map<U>(self, callback: impl FnOnce(R) -> U) -> WriteCallbackGuard<'a, S, U> {
        let Self { state, result } = self;
        WriteCallbackGuard {
            result: callback(result),
            state,
        }
    }
}
