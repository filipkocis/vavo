mod transform;
mod camera;
mod light;
mod face;

use glam::Vec2;
pub use transform::*;
pub use face::*;
pub use camera::*;
pub use light::*;

#[derive(crate::macros::Reflect)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn new_min_max(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x && point.y >= self.min.y && point.y <= self.max.y
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x < other.max.x && self.max.x > other.min.x && self.min.y < other.max.y && self.max.y > other.min.y
    }
}
