pub mod events;
pub mod event_handler;

use std::{any::{Any, TypeId}, collections::HashMap};

pub struct Events {
    /// Current frame events
    storage: HashMap<TypeId, Vec<Box<dyn Any>>>,
    /// Unapplied events, to be used in the next frame
    staging: HashMap<TypeId, Vec<Box<dyn Any>>>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            staging: HashMap::new(),
        }
    }

    /// Apply staged events to be used in the next frame
    pub(super) fn apply(&mut self) {
        self.storage.clear();

        for (type_id, events_staging) in self.staging.drain() {
            let events = self.storage.entry(type_id).or_insert(Vec::new());
            events.extend(events_staging);
        }
    }

    pub(super) fn write<T: 'static>(&mut self, resource: T) {
        let events_t = self.staging.entry(TypeId::of::<T>()).or_insert(Vec::new());
        events_t.push(Box::new(resource)); 
    }

    /// Write event T directly to the storage bypassing the staging area
    pub(super) fn write_immediately<T: 'static>(&mut self, resource: T) {
        let events_t = self.storage.entry(TypeId::of::<T>()).or_insert(Vec::new());
        events_t.push(Box::new(resource)); 
    }

    pub(super) fn read<T: 'static>(&self) -> Vec<&T> {
        if let Some(events_t) = self.storage.get(&TypeId::of::<T>()) {
            return events_t.iter().map(|e| 
                e.downcast_ref::<T>().unwrap()
            ).collect()
        }
        Vec::new()
    }
}
