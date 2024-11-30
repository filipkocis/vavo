use std::{any::{Any, TypeId}, collections::HashMap};

pub struct Events {
    storage: HashMap<TypeId, Vec<Box<dyn Any>>>,
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
    pub fn apply(&mut self) {
        self.storage.clear();

        for (type_id, events_staging) in self.staging.drain() {
            let events = self.storage.entry(type_id).or_insert(Vec::new());
            events.extend(events_staging);
        }
    }

    pub fn write<T: 'static>(&mut self, resource: T) {
        let events_t = self.staging.entry(TypeId::of::<T>()).or_insert(Vec::new());
        events_t.push(Box::new(resource)); 
    }

    pub fn read<T: 'static>(&self) -> Vec<&T> {
        if let Some(events_t) = self.storage.get(&TypeId::of::<T>()) {
            return events_t.iter().map(|e| 
                e.downcast_ref::<T>().unwrap()
            ).collect()
        }
        Vec::new()
    }
}

pub struct EventWriter<'a> {
    events: *mut Events,
    _marker: std::marker::PhantomData<&'a ()>,
}

pub struct EventReader<'a> {
    events: *const Events,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl Events {
    pub fn handlers<'a>(&'a mut self) -> (EventReader<'a>, EventWriter<'a>) {
        let reader = EventReader {
            events: self,
            _marker: std::marker::PhantomData,
        };
        let writer = EventWriter {
            events: self,
            _marker: std::marker::PhantomData,
        };

        (reader, writer)
    }
}

impl<'a> EventWriter<'a> {
    pub fn send<T: 'static>(&self, event: T) {
        unsafe {
            (*self.events).write(event);
        }
    }
}

impl<'a> EventReader<'a> {
    pub fn read<T: 'static>(&self) -> Vec<&T> {
        unsafe {
            (*self.events).read()
        }
    }
}
