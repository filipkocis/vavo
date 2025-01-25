use crate::app::input::{KeyCode, MouseButton};

pub use winit::event::{ElementState, MouseScrollDelta};

use glam::Vec2;

/// Event for keyboard input.
pub struct KeyboardInput {
    pub code: KeyCode,
    pub state: ElementState,
}

/// Event for mouse button input.
pub struct MouseInput {
    pub button: MouseButton,
    pub state: ElementState,
}

/// Event for mouse scroll wheel
pub struct MouseWheel {
    pub delta: MouseScrollDelta,
}

/// Event for mouse motion. Stores the delta of the mouse movement.
///
/// For absolute movement, use [`CursorMoved`](CursorMoved).
pub struct MouseMotion {
    pub delta: Vec2,
}

/// Event for cursor movement. Stores the absolute position of the cursor.
///
/// For relative movement, use [`MouseMotion`](MouseMotion).
pub struct CursorMoved {
    pub position: Vec2,
}
