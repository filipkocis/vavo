use glam::{Mat4, Vec3};
use vavo_macros::{Component, Reflect};

use super::{Sphere, AABB, OBB};

#[derive(Reflect, Component, Clone, Debug)]
/// A bounding volume that represents a world space bounding volume. Changes when the object's
/// [`GlobalTransform`](crate::math::GlobalTransform) changes, it is dependent on the
/// [`LocalBoundingVolume`](super::LocalBoundingVolume).
pub enum WorldBoundingVolume {
    Sphere(Sphere),
    AABB(AABB),
    OBB(OBB),
    None,
}

impl WorldBoundingVolume {
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

    /// Checks if two bounding volumes intersect
    pub fn intersects(&self, other: &Self) -> bool {
        use super::intersection::*;

        match (self, other) {
            (Self::Sphere(s1), Self::Sphere(s2)) => sphere_sphere(s1, s2),
            (Self::AABB(a1), Self::AABB(a2)) => aabb_aabb(a1, a2),
            (Self::OBB(o1), Self::OBB(o2)) => obb_obb(o1, o2),

            (Self::Sphere(s), Self::AABB(a)) |
            (Self::AABB(a), Self::Sphere(s)) => sphere_aabb(s, a),

            (Self::Sphere(s), Self::OBB(o)) |
            (Self::OBB(o), Self::Sphere(s)) => obb_sphere(o, s),

            (Self::AABB(a), Self::OBB(o)) |
            (Self::OBB(o), Self::AABB(a)) => obb_aabb(o, a),

            (Self::None, _) |
            (_, Self::None) => false,
        }
    }
}
