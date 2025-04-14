use glam::{Mat4, Vec3};
use vavo_macros::{Component, Reflect};

use super::{Sphere, AABB, OBB};

#[derive(Reflect, Component, Clone, Debug)]
/// A bounding volume that represents a local space bounding volume. Changes only when the object's
/// model changes. For world space bounding volumes, see
/// [`WorldBoundingVolume`](super::WorldBoundingVolume).
pub enum LocalBoundingVolume {
    Sphere(Sphere),
    AABB(AABB),
    OBB(OBB),
    None,
}

// TODO: refactor `to_**` methods to take a mesh, since doing a mut query on Change will be an
// infinite loop
impl LocalBoundingVolume {
    /// Creates a new bounding volume of type None
    pub fn new() -> Self {
        Self::None
    }

    pub fn new_sphere(center: Vec3, radius: f32) -> Self {
        Self::Sphere(Sphere::new(center, radius))
    }

    pub fn new_aabb(min: Vec3, max: Vec3) -> Self {
        Self::AABB(AABB::new(min, max))
    }

    pub fn new_obb(center: Vec3, half_extents: Vec3, rotation: Mat4) -> Self {
        Self::OBB(OBB::new(center, half_extents, rotation))
    }

    pub fn to_none(&mut self) {
        *self = Self::None;
    }
    
    /// Converts the bounding volume to a default sphere, if it is not already a sphere
    pub fn to_sphere(&mut self) {
        if let Self::Sphere(_) = self {
            return;
        }
        *self = Self::new_sphere(Vec3::ZERO, 0.0);
    }

    /// Converts the bounding volume to a default AABB, if it is not already an AABB
    pub fn to_aabb(&mut self) {
        if let Self::AABB(_) = self {
            return;
        }
        *self = Self::new_aabb(Vec3::ZERO, Vec3::ZERO);
    }

    /// Converts the bounding volume to a default OBB, if it is not already an OBB
    pub fn to_obb(&mut self) {
        if let Self::OBB(_) = self {
            return;
        }
        *self = Self::new_obb(Vec3::ZERO, Vec3::ZERO, Mat4::IDENTITY);
    }
}
