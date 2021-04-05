use std::*;

#[derive(Copy, Clone, Debug)]
pub enum Event {
    Initialization,
    Tick(time::Duration),
    Draw(time::Duration),
    Termination,
}

impl Event {
    /// Initialization event constructor
    pub fn initialization() -> Self {
        Event::Initialization
    }

    /// Tick event constructor
    pub fn tick() -> Self {
        Event::Tick(time::Duration::from_secs(0))
    }

    /// Draw event constructor
    pub fn draw() -> Self {
        Event::Draw(time::Duration::from_secs(0))
    }

    /// Termination event constructor
    pub fn termination() -> Self {
        Event::Termination
    }
}

impl hash::Hash for Event {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Event::Initialization => 0,
            Event::Tick(_) => 1,
            Event::Draw(_) => 2,
            Event::Termination => 3,
        }
        .hash(state)
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        use Event::*;
        match (self, other) {
            (Initialization, Initialization)
            | (Tick(_), Tick(_))
            | (Draw(_), Draw(_))
            | (Termination, Termination) => true,
            _ => false,
        }
    }
}

impl Eq for Event {}
