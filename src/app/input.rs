use std::{collections::HashSet, hash::Hash};

pub use winit::{
    keyboard::KeyCode, 
    event::{
        ElementState,
        MouseButton,
        MouseScrollDelta,
    },
};

use crate::{query::Query, system::{System, SystemStage, SystemsContext}};

use super::{App, Plugin};

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
        if !self.storage.contains(&key) {
            self.just_pressed.insert(key);
        }

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

/// UI input clearing system for just pressed inputs.
fn clear_just_pressed_inputs(ctx: &mut SystemsContext, _: Query<()>) {
    let resources = &mut ctx.resources;
    resources.get_mut::<Input<KeyCode>>().unwrap().clear_just_pressed();
    resources.get_mut::<Input<MouseButton>>().unwrap().clear_just_pressed();
}

/// Adds `Input<KeyCode>` and `Input<MouseButton>` resources to enable keyboard and mouse input
/// handling.
///
/// # Note
/// These can also be handled through events, by using `KeyboardInput` and `MouseInput` event types.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.world.resources.insert(Input::<KeyCode>::new());
        app.world.resources.insert(Input::<MouseButton>::new());

        app.register_system(
            System::new("clear_just_pressed_inputs", clear_just_pressed_inputs), 
            SystemStage::Last
        );
    }
}
