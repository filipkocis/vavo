use crate::{assets::Handle, renderer::{Color, palette, Image}};

use super::Rect;

/// Main camera component
/// Requires Projection, Transform, and Camera2D/3D components
pub struct Camera {
    pub active: bool,
    pub target: Option<Handle<Image>>,
    pub clear_color: Color,
}

/// Defines a 3D camera, required for 3D rendering
pub struct Camera3D {}

/// Projection type component, required for camera
pub enum Projection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection),
}

/// Used in Projection enum for camera
pub struct PerspectiveProjection {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

/// Used in Projection enum for camera
pub struct OrthographicProjection {
    pub area: Rect,
    pub scale: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            active: true,
            target: None,
            clear_color: palette::BLACK,
        }
    }
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {}
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            fov: 45.0,
            near: 0.1,
            far: 100.0,
            aspect_ratio: 1.0,
        }
    }
} 

impl Default for OrthographicProjection {
    fn default() -> Self {
        Self {
            area: Rect::new_min_max(-400., -300., 400., 300.),
            scale: 1.0,
            near: 0.1,
            far: 100.0,
        }
    }
}
