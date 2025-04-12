use glam::{Mat4, Vec3};
use vavo_macros::{Component, Reflect};

#[derive(Reflect, Component, Clone, Debug)]
/// Represents a bounding volume type, used for collision detection
pub enum BoundingVolume {
    Sphere(Sphere),
    AABB(AABB),
    OBB(OBB),
    Frustum(Frustum),
    None,
}

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

#[derive(Reflect, Clone, Debug)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn new(planes: [Plane; 6]) -> Self {
        Self { planes }
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

impl BoundingVolume {
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

    /// Creates a new bounding volume of type Frustum.
    /// Call [`Projection::get_frustum_planes`](crate::math::Projection::get_frustum_planes) to get the planes of the frustum. 
    pub fn new_frustum(planes: [Plane; 6]) -> Self {
        Self::Frustum(Frustum { planes })
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

    /// Checks if two bounding volumes intersect
    pub fn intersects(&self, other: &Self) -> bool {
        use intersection::*;

        match (self, other) {
            (Self::Sphere(s1), Self::Sphere(s2)) => sphere_sphere(s1, s2),
            (Self::AABB(a1), Self::AABB(a2)) => aabb_aabb(a1, a2),
            (Self::OBB(o1), Self::OBB(o2)) => obb_obb(o1, o2),
            (Self::Frustum(_f1), Self::Frustum(_f2)) => panic!("Frustum vs Frustum intersection not supported"), 

            (Self::Sphere(s), Self::AABB(a)) |
            (Self::AABB(a), Self::Sphere(s)) => sphere_aabb(s, a),
            
            (Self::Sphere(s), Self::Frustum(f)) |
            (Self::Frustum(f), Self::Sphere(s)) => frustum_sphere(f, s),

            (Self::AABB(a), Self::Frustum(f)) |
            (Self::Frustum(f), Self::AABB(a)) => frustum_aabb(f, a),

            (Self::OBB(o), Self::Frustum(f)) |
            (Self::Frustum(f), Self::OBB(o)) => frustum_obb(f, o),

            (Self::Sphere(s), Self::OBB(o)) |
            (Self::OBB(o), Self::Sphere(s)) => obb_sphere(o, s),

            (Self::AABB(a), Self::OBB(o)) |
            (Self::OBB(o), Self::AABB(a)) => obb_aabb(o, a),

            (Self::None, _) |
            (_, Self::None) => false,
        }
    }
}


pub mod intersection {
    //! Defines helper functions for intersection checking between 2 different bounding volume
    //! types

    use super::*;

    pub fn sphere_sphere(s1: &Sphere, s2: &Sphere) -> bool {
        let distance_squared = (s1.center - s2.center).length_squared();
        let radius_sum = s1.radius + s2.radius;
        distance_squared <= radius_sum * radius_sum
    }

    pub fn sphere_aabb(s: &Sphere, aabb: &AABB) -> bool {
        let closest_point = Vec3::new(
            s.center.x.clamp(aabb.min.x, aabb.max.x),
            s.center.y.clamp(aabb.min.y, aabb.max.y),
            s.center.z.clamp(aabb.min.z, aabb.max.z),
        );
        let distance_squared = (s.center - closest_point).length_squared();
        distance_squared <= s.radius * s.radius
    }

    pub fn aabb_aabb(a1: &AABB, a2: &AABB) -> bool {
        a1.min.x <= a2.max.x && a1.max.x >= a2.min.x &&
        a1.min.y <= a2.max.y && a1.max.y >= a2.min.y &&
        a1.min.z <= a2.max.z && a1.max.z >= a2.min.z
    }

    // Implement OBB vs OBB using the Separating Axis Theorem (SAT)
    pub fn obb_obb(o1: &OBB, o2: &OBB) -> bool {
        let axes1 = o1.get_obb_axes();
        let axes2 = o2.get_obb_axes();

        let mut axes = vec![];

        // 3 face normals from OBB1
        axes.extend_from_slice(&axes1);

        // 3 face normals from OBB2
        axes.extend_from_slice(&axes2);

        // 9 cross products
        for a in &axes1 {
            for b in &axes2 {
                let cross = a.cross(*b);
                if cross.length_squared() > 1e-6 {
                    axes.push(cross.normalize());
                }
            }
        }

        for axis in axes {
            if !o1.overlap_on_axis(o2, &axis) {
                return false; // Separating axis found
            }
        }

        true
    }

    pub fn obb_sphere(obb: &OBB, sphere: &Sphere) -> bool {
        // Transform sphere center into OBB local space
        let inv_rotation = obb.rotation.inverse();
        let local_center = inv_rotation.transform_point3(sphere.center - obb.center);

        // Clamp to extents
        let clamped = local_center.clamp(
            -obb.half_extents,
            obb.half_extents,
        );

        // Closest point in world space
        let closest = obb.center + obb.rotation.transform_vector3(clamped);

        // Check distance
        let distance_squared = (sphere.center - closest).length_squared();
        distance_squared <= sphere.radius * sphere.radius
    }

    pub fn obb_aabb(obb: &OBB, aabb: &AABB) -> bool {
        let center = aabb.center();
        let half_extents = aabb.half_extents();

        let aabb_as_obb = OBB {
            center,
            half_extents,
            rotation: Mat4::IDENTITY,
        };

        obb_obb(obb, &aabb_as_obb)
    }

    pub fn frustum_sphere(frustum: &Frustum, sphere: &Sphere) -> bool {
        for plane in &frustum.planes {
            let distance = plane.normal.dot(sphere.center) + plane.d;
            if distance < -sphere.radius {
                return false; // Sphere is outside the frustum
            }
        }
        true // Sphere is inside or intersecting the frustum
    }

    pub fn frustum_aabb(frustum: &Frustum, aabb: &AABB) -> bool {
        for plane in &frustum.planes {
            let mut p = Vec3::ZERO;
            if plane.normal.x > 0.0 {
                p.x = aabb.max.x;
            } else {
                p.x = aabb.min.x;
            }

            if plane.normal.y > 0.0 {
                p.y = aabb.max.y;
            } else {
                p.y = aabb.min.y;
            }

            if plane.normal.z > 0.0 {
                p.z = aabb.max.z;
            } else {
                p.z = aabb.min.z;
            }

            let distance = plane.normal.dot(p) + plane.d;
            if distance < 0.0 {
                return false; // AABB is outside the frustum
            }
        }
        true // AABB is inside or intersecting the frustum
    }

    pub fn frustum_obb(frustum: &Frustum, obb: &OBB) -> bool {
        // Check if OBB intersects frustum (simplified SAT approach)
        for plane in &frustum.planes {
            let mut p = Vec3::ZERO;
            if plane.normal.x > 0.0 {
                p.x = obb.center.x + obb.half_extents.x;
            } else {
                p.x = obb.center.x - obb.half_extents.x;
            }

            if plane.normal.y > 0.0 {
                p.y = obb.center.y + obb.half_extents.y;
            } else {
                p.y = obb.center.y - obb.half_extents.y;
            }

            if plane.normal.z > 0.0 {
                p.z = obb.center.z + obb.half_extents.z;
            } else {
                p.z = obb.center.z - obb.half_extents.z;
            }

            let distance = plane.normal.dot(p) + plane.d;
            if distance < 0.0 {
                return false; // OBB is outside the frustum
            }
        }
        true // OBB is inside or intersecting the frustum
    }
}
