use glam::Vec3;
use vavo_macros::{Component, Reflect};

use super::{intersection::{frustum_aabb, frustum_obb, frustum_sphere}, WorldBoundingVolume};

#[derive(Component, Reflect, Clone, Debug)]
/// A frustum is a bounding volume that represents a view frustum in 3D space. It's used directly,
/// not as a [`LocalBoundingVolume`](super::LocalBoundingVolume).
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Creates a new bounding volume of type Frustum.
    /// Call [`Projection::get_frustum_planes`](crate::math::Projection::get_frustum_planes) to get the planes of the frustum.
    pub fn new(planes: [Plane; 6]) -> Self {
        Self { planes }
    }

    /// Checks if a point is inside the frustum
    pub fn is_point_inside(&self, point: Vec3) -> bool {
        self.planes.iter().all(|plane| plane.is_point_in_front(point))
    }

    /// Checks if a bounding volume intersects with the frustum
    pub fn intersects(&self, volume: &WorldBoundingVolume) -> bool {
        match volume {
            WorldBoundingVolume::Sphere(sphere) => frustum_sphere(self, sphere),
            WorldBoundingVolume::AABB(aabb) => frustum_aabb(self, aabb),
            WorldBoundingVolume::OBB(obb) => frustum_obb(self, obb),
            WorldBoundingVolume::None => false,
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug)]
pub struct Plane {
    pub normal: Vec3,
    pub d: f32,
}

impl Plane {
    pub fn new(normal: Vec3, d: f32) -> Self {
        Self { normal, d }
    }

    /// Creates a plane from three points in 3D space
    pub fn from_points(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        let normal = (p2 - p1).cross(p3 - p1).normalize();
        let d = -normal.dot(p1);
        Self::new(normal, d)
    }

    /// Checks if a point is in front of the plane
    pub fn is_point_in_front(&self, point: Vec3) -> bool {
        self.normal.dot(point) + self.d > 0.0
    }

    /// Checks if a point is behind the plane
    pub fn is_point_behind(&self, point: Vec3) -> bool {
        self.normal.dot(point) + self.d < 0.0
    }

    /// Checks if a point is on the plane
    pub fn is_point_on(&self, point: Vec3) -> bool {
        self.normal.dot(point) + self.d == 0.0
    }
}
