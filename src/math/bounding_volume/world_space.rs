use glam::{Mat4, Vec3};
use vavo_macros::{Component, Reflect};

use super::{LocalBoundingVolume, Sphere, AABB, OBB};

#[derive(Default, Reflect, Component, Clone, Debug)]
/// A bounding volume that represents a world space bounding volume. Changes when the object's
/// [`GlobalTransform`](crate::math::GlobalTransform) changes, it is dependent on the
/// [`LocalBoundingVolume`](super::LocalBoundingVolume).
pub enum WorldBoundingVolume {
    Sphere(Sphere),
    AABB(AABB),
    OBB(OBB),
    #[default]
    None,
}

impl WorldBoundingVolume {
    /// Creates a new bounding volume of type None
    pub fn new() -> Self {
        Self::default()
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

/// A trait for converting a local space bounding volume to world space
pub trait ToWorldSpace {
    /// The output type of the conversion
    type Output;
    /// Converts the bounding volume to world space using the given global transform
    fn to_world_space(&self, transform: &Mat4) -> Self::Output;
}

impl ToWorldSpace for LocalBoundingVolume {
    type Output = WorldBoundingVolume;

    fn to_world_space(&self, transform: &Mat4) -> Self::Output {
        match self {
            Self::Sphere(sphere) => WorldBoundingVolume::Sphere(sphere.to_world_space(transform)),
            Self::AABB(aabb) => WorldBoundingVolume::AABB(aabb.to_world_space(transform)),
            Self::OBB(obb) => WorldBoundingVolume::OBB(obb.to_world_space(transform)),
            Self::None => WorldBoundingVolume::None,
        }
    }
}

impl ToWorldSpace for Sphere {
    type Output = Sphere;

    fn to_world_space(&self, transform: &Mat4) -> Self::Output {
        let center = transform.transform_point3(self.center);
        let scale = transform.to_scale_rotation_translation().0;
        // Assuming non-uniform scaling, but we take the max scale to be conservative
        let radius = self.radius * scale.abs().max_element(); 

        Self { center, radius }
    }
}

impl ToWorldSpace for AABB {
    type Output = AABB;

    fn to_world_space(&self, transform: &Mat4) -> Self::Output {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let transformed = corners.map(|corner| transform.transform_point3(corner));

        let mut min = transformed[0];
        let mut max = transformed[0];

        for &point in &transformed[1..] {
            min = min.min(point);
            max = max.max(point);
        }

        Self { min, max }
    }
}

impl ToWorldSpace for OBB {
    type Output = OBB;

    fn to_world_space(&self, transform: &Mat4) -> Self::Output {
        let center = transform.transform_point3(self.center);
        let (scale, rotation_quat, _) = transform.to_scale_rotation_translation();
        let half_extents = self.half_extents * scale.abs();
        let rotation = Mat4::from_quat(rotation_quat) * self.rotation;

        Self {
            center,
            half_extents,
            rotation,
        }
    }
}
