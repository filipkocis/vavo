use std::{collections::HashSet, hash::Hash};

pub use winit::{keyboard::KeyCode, event::MouseButton};

use crate::{query::Query, system::{SystemStage, SystemsContext}};

use super::{App, Plugin};

/// A type which can be used as input data in the [`Input`](Input) resource.
trait InputData: Eq + Hash + Copy + Send + Sync + 'static {}

impl InputData for KeyCode {}
impl InputData for MouseButton {}

#[allow(private_bounds)]
#[derive(Debug, crate::macros::Resource)]
pub struct Input<I: InputData> {
    storage: HashSet<I>,
    just_pressed: HashSet<I>,
}

impl<I: InputData> Default for Input<I> {
    fn default() -> Self {
        Self {
            storage: HashSet::new(),
            just_pressed: HashSet::new(),
        }
    }
}

#[allow(private_bounds)]
impl<I: InputData> Input<I> {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn press(&mut self, key: I) {
        if !self.storage.contains(&key) {
            self.just_pressed.insert(key);
        }

        self.storage.insert(key);
    }

    pub(crate) fn release(&mut self, key: I) {
        self.storage.remove(&key);
    }

    pub(crate) fn clear_just_pressed(&mut self) {
        self.just_pressed.clear();
    }

    pub fn pressed(&self, key: I) -> bool {
        self.storage.contains(&key)
    }

    pub fn pressed_any(&self, keys: &[I]) -> bool {
        keys.iter().any(|key| self.pressed(*key))
    }

    pub fn pressed_all(&self, keys: &[I]) -> bool {
        keys.iter().all(|key| self.pressed(*key))
    }

    pub fn just_pressed(&self, key: I) -> bool {
        self.just_pressed.contains(&key)
    }
}

/// UI input clearing system for just pressed inputs.
fn clear_just_pressed_inputs(ctx: &mut SystemsContext, _: Query<()>) {
    let resources = &mut ctx.resources;
    resources.get_mut::<Input<KeyCode>>().clear_just_pressed();
    resources.get_mut::<Input<MouseButton>>().clear_just_pressed();
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

        app.register_system(clear_just_pressed_inputs, SystemStage::Last);
    }
}
