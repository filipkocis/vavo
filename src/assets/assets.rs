use std::collections::HashMap;

use super::Handle;

pub struct Assets<T> {
    storage: HashMap<Handle<T>, T>,
    next_id: u64,
}

impl<T> Assets<T> {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            next_id: 0,
        }
    }

    fn step_id(&mut self) -> Handle<T> {
        let id = self.next_id;
        self.next_id += 1;
        Handle::new(id)
    }

    pub fn add(&mut self, asset: T) -> Handle<T> {
        let id = self.step_id();
        self.storage.insert(id.clone(), asset);
        id
    }

    pub fn get(&self, id: &Handle<T>) -> Option<&T> {
        self.storage.get(id)
    }

    pub fn get_mut(&mut self, id: &Handle<T>) -> Option<&mut T> {
        self.storage.get_mut(id)
    }

    pub fn remove(&mut self, id: &Handle<T>) -> Option<T> {
        self.storage.remove(id)
    }
}
