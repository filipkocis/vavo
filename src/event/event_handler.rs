use crate::{
    event::Event,
    prelude::{Res, ResMut},
};

use super::Events;

/// Event handler for writing new events
pub struct EventWriter<E: Event> {
    events: ResMut<Events<E>>,
}

/// Event handler for reading events
pub struct EventReader<E: Event> {
    events: Res<Events<E>>,
}

impl<E: Event> EventWriter<E> {
    /// Get a reader for reading events
    #[inline]
    pub(crate) fn new(events: ResMut<Events<E>>) -> EventWriter<E> {
        EventWriter { events }
    }

    /// Write a new event
    #[inline]
    pub fn write(&mut self, event: E) {
        self.events.write(event);
    }
}

impl<E: Event> EventReader<E> {
    /// Get a reader for reading events
    #[inline]
    pub(crate) fn new(events: Res<Events<E>>) -> EventReader<E> {
        EventReader { events }
    }

    /// Read all events of type E
    #[inline]
    pub fn read(&self) -> &[E] {
        self.events.read()
    }

    /// Check if any events of type E exist
    #[inline]
    pub fn has_any(&self) -> bool {
        !self.events.is_empty()
    }

    /// Check if no events of type E exist
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
