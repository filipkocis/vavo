use std::marker::PhantomData;

use super::Events;

/// Event handler for sending new events
pub struct EventWriter<'a> {
    events: *mut Events,
    _marker: PhantomData<&'a ()>,
}

/// Event handler for reading events
pub struct EventReader<'a> {
    events: *const Events,
    _marker: PhantomData<&'a ()>,
}

impl<'a> EventWriter<'a> {
    /// Send a new event
    #[inline]
    pub fn send<T: 'static>(&self, event: T) {
        unsafe {
            (*self.events).write(event);
        }
    }
}

impl<'a> EventReader<'a> {
    /// Read all events of type T
    #[inline]
    pub fn read<T: 'static>(&self) -> Vec<&T> {
        unsafe { (*self.events).read() }
    }
}

impl Events {
    /// Wrap Events in read and write handlers
    #[inline]
    pub(crate) fn handlers<'a>(&'a mut self) -> (EventReader<'a>, EventWriter<'a>) {
        let reader = EventReader {
            events: self,
            _marker: PhantomData,
        };
        let writer = EventWriter {
            events: self,
            _marker: PhantomData,
        };

        (reader, writer)
    }
}
