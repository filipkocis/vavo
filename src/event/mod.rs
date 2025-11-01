mod event_handler;
mod events;
pub mod plugin;

pub use event_handler::*;
pub use events::*;

/// Manager for events. Created events are stored in a staging area until the end of the frame.
#[derive(Debug, crate::macros::Resource)]
pub struct Events<E: Event> {
    /// Double buffer for events
    /// storage: Current frame events
    /// staging: Unapplied events, to be used in the next frame
    buffers: [Vec<E>; 2],
    /// Indicates if the storage and stagging buffers have been swapped. If false, storage will be
    /// at index 0, otherwise at index 1
    swapped: bool,
}

impl<E: Event> Default for Events<E> {
    fn default() -> Self {
        Self {
            buffers: [Vec::new(), Vec::new()],
            swapped: false,
        }
    }
}

impl<E: Event> Events<E> {
    /// Create new empty event manager
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the storage buffer index
    #[inline]
    fn storage(&self) -> usize {
        if self.swapped { 1 } else { 0 }
    }

    /// Get the staging buffer index
    #[inline]
    fn staging(&self) -> usize {
        if self.swapped { 0 } else { 1 }
    }

    /// Apply staged events to be used in the next frame
    #[inline]
    pub(super) fn apply(&mut self) {
        let storage = self.storage();

        self.buffers[storage].clear();

        self.swapped = !self.swapped;
    }

    /// Write event `E` to the staging area
    pub(super) fn write(&mut self, event: E) {
        let staging = self.staging();
        let staging = &mut self.buffers[staging];
        staging.push(event);
    }

    /// Read all events of type `E` from the storage
    pub(super) fn read(&self) -> &[E] {
        let storage = self.storage();
        let storage = &self.buffers[storage];
        storage.as_slice()
    }

    /// Check if events of type `E` are empty
    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.buffers[self.storage()].is_empty()
    }
}
