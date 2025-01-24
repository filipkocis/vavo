use std::collections::HashMap;

use super::Handle;

/// Storage for assets of the same type accessible by their handle
pub struct Assets<T> {
    storage: HashMap<Handle<T>, T>,
    next_id: u64,
}

impl<T> Assets<T> {
    /// Create new empty asset storage
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

    /// Adds new asset to the storage and returns its handle
    pub fn add(&mut self, asset: T) -> Handle<T> {
        let id = self.step_id();
        self.storage.insert(id.clone(), asset);
        id
    }

    /// Inserts asset with the given handle, if the handle is already in use, it will be
    /// overwritten
    pub fn insert(&mut self, id: Handle<T>, asset: T) {
        self.storage.insert(id, asset);
    }

    /// Get a reference to the asset
    pub fn get(&self, id: &Handle<T>) -> Option<&T> {
        self.storage.get(id)
    }

    /// Get a mutable reference to the asset
    pub fn get_mut(&mut self, id: &Handle<T>) -> Option<&mut T> {
        self.storage.get_mut(id)
    }

    /// Removes and returns the asset
    pub fn remove(&mut self, id: &Handle<T>) -> Option<T> {
        self.storage.remove(id)
    }
}
