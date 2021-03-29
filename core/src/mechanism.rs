use super::*;
use itertools::*;

pub trait Mechanism<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    fn name(&self) -> &'static str;
    fn tick(&mut self, state: &mut S, event: E);
}

pub trait ReadMechanism<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    fn name(&self) -> &'static str;
    fn tick(&mut self, state: &S, event: E);
}

pub struct Mechanisms<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    all_mechanisms: Vec<Box<dyn Mechanism<S, E>>>,
    events_to_mechanisms: collections::HashMap<E, Vec<usize>>,
    all_read_mechanisms: Vec<Box<dyn ReadMechanism<S, E>>>,
    events_to_read_mechanisms: collections::HashMap<E, Vec<usize>>,
}

impl<S, E> Mechanisms<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    pub fn tick_event(&mut self, state: &mut S, event: E) {
        let Self {
            all_mechanisms,
            events_to_mechanisms,
            all_read_mechanisms,
            events_to_read_mechanisms,
        } = self;
        events_to_mechanisms.get(&event).map_or((), |ids| {
            ids.iter().cloned().for_each(|id| unsafe {
                all_mechanisms
                    .get_unchecked_mut(id)
                    .tick(state, event.clone())
            })
        });
        events_to_read_mechanisms.get(&event).map_or((), |ids| {
            ids.iter().cloned().for_each(|id| unsafe {
                all_read_mechanisms
                    .get_unchecked_mut(id)
                    .tick(state, event.clone())
            })
        });
    }
}

pub(crate) struct MechanismsBuilder<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    all_mechanisms: Vec<Box<dyn Mechanism<S, E>>>,
    events_to_mechanisms: collections::HashMap<E, Vec<usize>>,
    all_read_mechanisms: Vec<Box<dyn ReadMechanism<S, E>>>,
    events_to_read_mechanisms: collections::HashMap<E, Vec<usize>>,
}

impl<'a, S, E> MechanismsBuilder<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    pub fn with_mechanism(
        mut self,
        mechanism: impl Mechanism<S, E> + 'static,
        events: impl IntoIterator<Item = E>,
    ) -> Self {
        let Self {
            all_mechanisms,
            events_to_mechanisms,
            ..
        } = &mut self;
        let id = all_mechanisms.len();

        all_mechanisms.push(Box::new(mechanism));
        events
            .into_iter()
            .unique()
            .for_each(|event| events_to_mechanisms.entry(event).or_default().push(id));

        self
    }

    pub fn with_read_mechanism(
        mut self,
        read_mechanism: impl ReadMechanism<S, E> + 'static,
        events: impl IntoIterator<Item = E>,
    ) -> Self {
        let Self {
            all_read_mechanisms,
            events_to_read_mechanisms,
            ..
        } = &mut self;
        let id = all_read_mechanisms.len();

        all_read_mechanisms.push(Box::new(read_mechanism));
        events
            .into_iter()
            .unique()
            .for_each(|event| events_to_read_mechanisms.entry(event).or_default().push(id));

        self
    }

    pub fn build(self) -> Result<Mechanisms<S, E>, &'static str> {
        let Self {
            all_mechanisms,
            events_to_mechanisms,
            all_read_mechanisms,
            events_to_read_mechanisms,
        } = self;
        Ok(Mechanisms {
            all_mechanisms,
            events_to_mechanisms,
            all_read_mechanisms,
            events_to_read_mechanisms,
        })
    }
}

impl<S, E> Default for MechanismsBuilder<S, E>
where
    S: ClockworkState,
    E: ClockworkEvent,
{
    fn default() -> Self {
        MechanismsBuilder {
            all_mechanisms: Default::default(),
            events_to_mechanisms: Default::default(),
            all_read_mechanisms: Default::default(),
            events_to_read_mechanisms: Default::default(),
        }
    }
}
