use std::collections::HashMap;

use super::{Asset, Handle};
use crate::macros::Resource;

/// Storage for assets of the same type accessible by their handle
#[derive(Resource)]
pub struct Assets<A: Asset> {
    storage: HashMap<Handle<A>, A>,
    next_id: u64,
}

impl<A: Asset> Assets<A> {
    /// Create new empty asset storage
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            next_id: 0,
        }
    }

    fn step_id(&mut self) -> Handle<A> {
        let id = self.next_id;
        self.next_id += 1;
        Handle::new(id)
    }

    /// Adds new asset to the storage and returns its handle
    pub fn add(&mut self, asset: A) -> Handle<A> {
        let id = self.step_id();
        self.storage.insert(id.clone(), asset);
        id
    }

    /// Inserts asset with the given handle, if the handle is already in use, it will be
    /// overwritten
    pub fn insert(&mut self, id: Handle<A>, asset: A) {
        self.storage.insert(id, asset);
    }

    /// Get a reference to the asset
    pub fn get(&self, id: &Handle<A>) -> Option<&A> {
        self.storage.get(id)
    }

    /// Get a mutable reference to the asset
    pub fn get_mut(&mut self, id: &Handle<A>) -> Option<&mut A> {
        self.storage.get_mut(id)
    }

    /// Removes and returns the asset
    pub fn remove(&mut self, id: &Handle<A>) -> Option<A> {
        self.storage.remove(id)
    }
}
