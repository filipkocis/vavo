pub mod intersection;
mod world_space;
mod local_space;
mod frustum;

pub use local_space::*;
pub use world_space::*;
pub use frustum::*;

use glam::{Mat4, Vec3};
use vavo_macros::Reflect;

use crate::prelude::Mesh;

#[derive(Reflect, Clone, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn diameter(&self) -> f32 {
        self.radius * 2.0
    }

    pub fn volume(&self) -> f32 {
        (4.0 / 3.0) * std::f32::consts::PI * self.radius.powi(3)
    }

    pub fn surface_area(&self) -> f32 {
        4.0 * std::f32::consts::PI * self.radius.powi(2)
    }

    /// Calculates the bounding sphere of a mesh
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let center = mesh.center();
        let radius = mesh.max_distance(); 

        Self {
            center,
            radius,
        }
    }

    /// Returns the bounding sphere in world space
    pub fn to_world_space(&self, transform: &Mat4) -> Self {
        let center = transform.transform_point3(self.center);
        let scale = transform.to_scale_rotation_translation().0;
        // Assuming non-uniform scaling, but we take the max scale to be conservative
        let radius = self.radius * scale.abs().max_element(); 

        Self { center, radius }
    }
}

#[derive(Reflect, Clone, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Calculates the bounding AABB of a mesh
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let (min, max) = mesh.min_max_bounds(); 

        Self { min, max }
    }

    /// Returns the bounding AABB in world space
    pub fn to_world_space(&self, transform: &Mat4) -> Self {
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

#[derive(Reflect, Clone, Debug)]
pub struct OBB {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub rotation: Mat4,
}

impl OBB {
    pub fn new(center: Vec3, half_extents: Vec3, rotation: Mat4) -> Self {
        Self {
            center,
            half_extents,
            rotation,
        }
    }

    /// Calculates the bounding OBB of a mesh
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let (min, max) = mesh.min_max_bounds();
        let center = (min + max) * 0.5;
        let half_extents = (max - min) * 0.5;
        let rotation = Mat4::IDENTITY;

        Self {
            center,
            half_extents,
            rotation,
        }
    }

    /// Returns the bounding OBB in world space
    pub fn to_world_space(&self, transform: &Mat4) -> Self {
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

    /// Returns the axes of the OBB in world space
    pub fn get_obb_axes(&self) -> [Vec3; 3] {
        [
            self.rotation.col(0).truncate().normalize(),
            self.rotation.col(1).truncate().normalize(),
            self.rotation.col(2).truncate().normalize(),
        ]
    }

    /// Returns the corners of the OBB in world space
    pub fn get_obb_corners(&self) -> Vec<Vec3> {
        let he = self.half_extents;
        let signs = [-1.0, 1.0];
        let mut corners = Vec::with_capacity(8);

        for &x in &signs {
            for &y in &signs {
                for &z in &signs {
                    let local = Vec3::new(x * he.x, y * he.y, z * he.z);
                    let rotated = self.rotation.mul_vec4(local.extend(1.0)).truncate();
                    corners.push(self.center + rotated);
                }
            }
        }

        corners
    }

    /// Projects the OBB onto a given axis and returns the min and max values
    pub fn project_obb(&self, axis: Vec3) -> (f32, f32) {
        let corners = self.get_obb_corners();
        let mut min = axis.dot(corners[0]);
        let mut max = min;

        for corner in &corners[1..] {
            let projection = axis.dot(*corner);
            min = min.min(projection);
            max = max.max(projection);
        }

        (min, max)
    }

    /// Checks if the OBB overlaps with another OBB on a given axis
    pub fn overlap_on_axis(&self, other: &OBB, axis: &Vec3) -> bool {
        let (min1, max1) = self.project_obb(*axis);
        let (min2, max2) = other.project_obb(*axis);

        max1 >= min2 && max2 >= min1
    }
}
