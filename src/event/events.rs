use crate::{
    app::input::{KeyCode, MouseButton},
    event::Events,
    prelude::ResMut,
};

use crate::macros::Event;
pub use winit::event::{DeviceEvent, WindowEvent};
pub use winit::event::{ElementState, MouseScrollDelta};

use glam::Vec2;

/// System to apply all staged events
pub fn apply_events<E: Event>(mut events: ResMut<Events<E>>) {
    events.apply();
}

/// Marker trait for events
pub trait Event: Send + Sync + 'static {}

impl Event for DeviceEvent {}
impl Event for WindowEvent {}

/// Event for keyboard input.
#[derive(Event)]
pub struct KeyboardInput {
    pub code: KeyCode,
    pub state: ElementState,
}

/// Event for mouse button input.
#[derive(Event)]
pub struct MouseInput {
    pub button: MouseButton,
    pub state: ElementState,
}

/// Event for mouse scroll wheel
#[derive(Event)]
pub struct MouseWheel {
    pub delta: MouseScrollDelta,
}

/// Event for mouse motion. Stores the delta of the mouse movement.
///
/// For absolute movement, use [`CursorMoved`](CursorMoved).
#[derive(Event)]
pub struct MouseMotion {
    pub delta: Vec2,
}

/// Event for cursor movement. Stores the absolute position of the cursor.
///
/// For relative movement, use [`MouseMotion`](MouseMotion).
#[derive(Event)]
pub struct CursorMoved {
    pub position: Vec2,
}
