use std::{collections::HashSet, hash::Hash};

pub use winit::{
    keyboard::KeyCode, 
    event::{
        ElementState,
        MouseButton,
        MouseScrollDelta,
    },
};

pub struct Input<T> 
where T: Eq + Hash + Copy
{
    storage: HashSet<T>,
    just_pressed: HashSet<T>,
}

impl<T> Input<T> 
where T: Eq + Hash + Copy
{
    pub fn new() -> Self {
        Self {
            storage: HashSet::new(),
            just_pressed: HashSet::new(),
        }
    }

    pub(crate) fn press(&mut self, key: T) {
        self.storage.insert(key);
    }

    pub(crate) fn release(&mut self, key: T) {
        self.storage.remove(&key);
    }

    pub(crate) fn clear_just_pressed(&mut self) {
        self.just_pressed.clear();
    }

    pub fn pressed(&self, key: T) -> bool {
        self.storage.contains(&key)
    }

    pub fn pressed_any(&self, keys: &[T]) -> bool {
        keys.iter().any(|key| self.pressed(*key))
    }

    pub fn pressed_all(&self, keys: &[T]) -> bool {
        keys.iter().all(|key| self.pressed(*key))
    }

    pub fn just_pressed(&self, key: T) -> bool {
        self.just_pressed.contains(&key)
    }
}
