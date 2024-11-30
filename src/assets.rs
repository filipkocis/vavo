use std::{collections::HashMap, fmt::Debug, hash::Hash};

pub struct Assets<T> {
    storage: HashMap<AssetHandle<T>, T>,
    next_id: u64,
}

pub struct AssetHandle<T> {
    id: u64,
    _marker: std::marker::PhantomData<T>,
}
impl<T> AssetHandle<T> {
    fn new(id: u64) -> Self {
        Self {
            id,
            _marker: std::marker::PhantomData
        }
    }
}
impl<T> Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for AssetHandle<T> {}
impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self::new(self.id)
    }
}
impl<T> Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssetHandle({})", self.id)
    }
}

impl<T> Assets<T> {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            next_id: 0,
        }
    }

    fn step_id(&mut self) -> AssetHandle<T> {
        let id = self.next_id;
        self.next_id += 1;
        AssetHandle::new(id)
    }

    pub fn add(&mut self, asset: T) -> AssetHandle<T> {
        let id = self.step_id();
        self.storage.insert(id.clone(), asset);
        id
    }

    pub fn get(&self, id: AssetHandle<T>) -> Option<&T> {
        self.storage.get(&id)
    }

    pub fn get_mut(&mut self, id: AssetHandle<T>) -> Option<&mut T> {
        self.storage.get_mut(&id)
    }

    pub fn remove(&mut self, id: AssetHandle<T>) -> Option<T> {
        self.storage.remove(&id)
    }
}
