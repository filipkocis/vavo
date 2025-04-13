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
