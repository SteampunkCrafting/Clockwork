use std::fmt::Debug;
use std::hash::Hash;

/// A set of constraints, which every valid Clockwork state should satisfy.
pub trait ClockworkState: Sized + 'static {}

/// A set of constraints, which every valid Clockwork event type should satisfy.
pub trait ClockworkEvent: Send + Clone + Eq + Hash + Debug + 'static {}
impl<T> ClockworkEvent for T where T: Send + Clone + Eq + Hash + Debug + 'static {}

/// A substate of a clockwork state.
///
/// In order to provide statically-resolved, yet generic system of states
/// (so that we can toss around mechanisms), we require to heavily rely on
/// delegations. For this purpose, a `Substate` trait is being introduced.
///
/// Being a `Substate<S>` basically means one of:
/// - Being `S`
/// - Containing `S`
///
/// (Remember that having these two conditions satisfied is a logical contradiction)
///
/// A first thing to notice is that a valid `ClockworkState` `S` automatically
/// implies `Substate<S>`:
///
/// ```
/// # use spc_clockwork_kernel::prelude::*;
/// // Assume that a certain clockwork mechanism requires an i32 counter
/// struct A(i32);
/// let a: &A = A(0).substate();
/// let a_mut: &mut A = A(0).substate_mut();
/// ```
///
/// This means that if there is no composite state in our program, we do not have to implement
/// substates to utilize generic mechanisms.
///
/// Obviously, there would be no point in using substates, if we didn't compose them:
/// ```
/// # use spc_clockwork_kernel::prelude::*;
/// /* ---- Event section ---- */
/// type Event = ();
///
/// /* ---- State section ---- */
/// struct A(i32); // A state of one mechanism
///
/// struct B(Result<f32, &'static str>); // A state of another mechanism
///
/// struct S(A, B); // A combined state of both mechanisms
/// impl Substate<A> for S {
///     fn substate(&self) -> &A { self.0.substate() }
///     fn substate_mut(&mut self) -> &mut A { self.0.substate_mut() }
/// }
/// impl Substate<B> for S {
///     fn substate(&self) -> &B { self.1.substate() }
///     fn substate_mut(&mut self) -> &mut B { self.1.substate_mut() }
/// }
///
/// /* ---- Mechanism section ---- */
/// struct MechanismA;
/// impl<S> Mechanism<S, Event> for MechanismA
/// where
///     S: Substate<A>
/// {
///     fn name(&self) -> &'static str { "Mechanism A" }
///     fn clink(&mut self,state: &mut S,event: ()) {
///         let state: &mut A = state.substate_mut();
///         // ...
///     }
/// }
///
/// struct MechanismB;
/// impl<S> ReadMechanism<S, Event> for MechanismB
/// where
///     S: Substate<B>
/// {
///     fn name(&self) -> &'static str { "Mechanism B" }
///     fn clink(&mut self, state: &S, event: ()) {
///         let state: &B = state.substate();
///         // ...
///     }
/// }
///
/// /* ---- Main loop section ---- */
/// let mut state: S = S( // Creating state
///     A(128),
///     B(Ok(3.14))
/// );
///
/// let mut mechanism_a = MechanismA;
/// let mut mechanism_b = MechanismB;
///
/// mechanism_a.clink(&mut state, ());
/// mechanism_b.clink(&state, ());
/// ```
///
/// Note: this chaining of states may produce a lot of unnecessary code due
/// to lots of trait delegations. In the future, this should be overcome by
/// providing macros.
pub trait Substate<S>: CallbackSubstate<S> + ClockworkState
where
    S: ClockworkState + ?Sized,
{
    /// Gets an immutable reference to the substate
    fn substate(&self) -> &S;
    /// Gets a mutable reference to the substate
    fn substate_mut(&mut self) -> &mut S;
}
impl<S> Substate<S> for S
where
    S: ClockworkState,
{
    fn substate(&self) -> &S {
        self
    }

    fn substate_mut(&mut self) -> &mut S {
        self
    }
}

/// A substate of a clockwork state, accessible via callback.
///
/// This is a more general type trait, which should be requested by
/// mechanisms.
pub trait CallbackSubstate<S>
where
    S: ClockworkState + ?Sized,
{
    /// Executes provided callback, supplying its substate reference
    fn callback_substate<R>(&self, callback: impl FnOnce(&S) -> R) -> R;
    /// Executes provided callback, supplying its mutable substate reference
    fn callback_substate_mut<R>(&mut self, callback: impl FnOnce(&mut S) -> R) -> R;
}

/// Having substate always implies a possibility
/// to execute a callback on this substate.
/// This is not always true the other way.
impl<T, S> CallbackSubstate<S> for T
where
    T: Substate<S>,
    S: ClockworkState + ?Sized,
{
    fn callback_substate<R>(&self, callback: impl FnOnce(&S) -> R) -> R {
        callback(self.substate())
    }

    fn callback_substate_mut<R>(&mut self, callback: impl FnOnce(&mut S) -> R) -> R {
        callback(self.substate_mut())
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
    /// allowing read-only access to the substate objects
    /// of this clockwork state.
    pub fn get(&self) -> ReadCallbackGuard<'_, S> {
        ReadCallbackGuard {
            state: self,
            result: (),
        }
    }

    /// Constructs a write callback guard,
    /// allowing read-write access to the substate objects
    /// of this clockwork state.
    pub fn get_mut<T, R>(&mut self) -> WriteCallbackGuard<'_, S>
    where
        T: ClockworkState,
        S: CallbackSubstate<T>,
    {
        WriteCallbackGuard {
            state: self,
            result: (),
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
    pub fn then_get<T, U>(self, callback: impl FnOnce(&T, R) -> U) -> ReadCallbackGuard<'a, S, U>
    where
        T: ClockworkState,
        S: CallbackSubstate<T>,
    {
        let Self { state, result } = self;
        ReadCallbackGuard {
            result: state
                .0
                .callback_substate(move |state| callback(state, result)),
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
        S: CallbackSubstate<T>,
    {
        let Self { state, result } = self;
        WriteCallbackGuard {
            result: state
                .0
                .callback_substate(move |state| callback(result, state)),
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
        S: CallbackSubstate<T>,
    {
        let Self { state, result } = self;
        WriteCallbackGuard {
            result: state
                .0
                .callback_substate_mut(move |state| callback(result, state)),
            state,
        }
    }
}
