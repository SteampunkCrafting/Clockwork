use crate::abstract_runtime::{ClockworkState, ClockworkStateRequirements};

/// Initializable state
///
/// Represents an object, which has three main stages:
/// 1. Uninitialized -- when the state is not ready to be used,
///    and has to be partially initialized at runtime.
/// 2. Initialized -- when the state is ready to be used.
/// 3. Terminated -- when the state has been disposed,
///    and cannot be used anymore.
///
/// These stages follow each other during the application runtime.
#[derive(Clone, Debug)]
pub enum InitState<U, T> {
    /// Uninitialized state
    Uninit(U),

    /// Initialized state
    Init(T),

    /// Terminated state
    Terminated,
}

/// An interface for the Uninit variant
impl<U, T> InitState<U, T> {
    /// Initializes the state, given a mapping closure from the
    /// uninitialized state to the initialized state.
    ///
    /// > The enum variant is replaced from Uninit to Init.
    ///
    /// # Panics
    /// Panics if the state is already initialized.
    pub fn initialize(&mut self, callback: impl FnOnce(U) -> T) {
        *self = InitState::Init(match std::mem::replace(self, InitState::Terminated) {
            InitState::Uninit(inner) => callback(inner),
            other => other.on_bad_access("Tried to initialize the state"),
        })
    }

    /// Returns an immutable reference to the inner state, if the state is uninitialized.
    ///
    /// # Panics
    /// Panics if the state has been initialized.
    pub fn get_uninit(&self) -> &U {
        match self {
            InitState::Uninit(x) => x,
            _ => self.on_bad_access("Tried to read the uninitialized state"),
        }
    }

    /// Returns a mutable reference to the inner state, if the state is uninitialized.
    ///
    /// # Panics
    /// Panics if the state has been initialized.
    pub fn get_uninit_mut(&mut self) -> &mut U {
        match self {
            InitState::Uninit(x) => x,
            _ => self.on_bad_access("Tried to write to the uninitialized state"),
        }
    }
}

/// An interface for Init variant
impl<U, T> InitState<U, T>
where
    T: Sized,
    U: Sized,
{
    /// Terminates the state, given some callback closure for state disposal.
    ///
    /// > The enum variant is replaced from Init to Terminated.
    ///
    /// # Panics
    /// Panics if the state is already initialized.
    pub fn terminate(&mut self, callback: impl FnOnce(T)) {
        match std::mem::replace(self, InitState::Terminated) {
            InitState::Init(x) => callback(x),
            _ => self.on_bad_access("Tried to terminate the state"),
        };
    }

    /// Returns an immutable reference to the inner state, if the state is initialized.
    ///
    /// # Panics
    /// Panics if the state has been terminated, or has not been initialized.
    pub fn get_init(&self) -> &T {
        match self {
            InitState::Init(x) => x,
            _ => self.on_bad_access("Tried to read the state"),
        }
    }

    /// Returns a mutable reference to the inner state, if the state is initialized.
    ///
    /// # Panics
    /// Panics if the state has been terminated, or has not been initialized.
    pub fn get_init_mut(&mut self) -> &mut T {
        match self {
            InitState::Init(x) => x,
            _ => self.on_bad_access("Tried to write to the state"),
        }
    }
}

/// Misc
impl<U, T> InitState<U, T> {
    /// Raises an error depending on the enum variant and the action.
    #[inline(always)]
    fn on_bad_access<V>(&self, action_msg: &'static str) -> V {
        match self {
            InitState::Uninit(_) => {
                panic!("{}, but the state has not yet been initialized", action_msg)
            }
            InitState::Init(_) => {
                panic!("{}, but the state has already been initialized", action_msg)
            }
            InitState::Terminated => {
                panic!("{}, but the state has already been terminated", action_msg)
            }
        }
    }
}

impl<U, T> Default for InitState<U, T>
where
    U: Default,
{
    fn default() -> Self {
        InitState::Uninit(Default::default())
    }
}

impl<U, T> ClockworkState for InitState<U, T>
where
    U: ClockworkStateRequirements,
    T: ClockworkStateRequirements,
{
}

impl<U, T> From<U> for InitState<U, T> {
    fn from(x: U) -> Self {
        InitState::Uninit(x)
    }
}
