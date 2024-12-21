use std::ops::BitOr;

use glam::{Mat4, Quat, Vec3};

use crate::{palette, prelude::Color};

pub enum CubeMapFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

enum LightFlags {
    Ambient = 0,
    Directional = 1,
    Point = 2,
    Spot = 3,
    Visible = 4,
    CastShadow = 5,
}

impl BitOr for LightFlags {
    type Output = u32;

    fn bitor(self, rhs: Self) -> Self::Output {
        1 << self as u32 | 1 << rhs as u32
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    pub color: [f32; 4],
    pub intensity: f32,
    pub flags: u32,
    pub range: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
    _padding: [f32; 3],
    pub view_proj: [[f32; 4]; 4],
} 

/// Ambient light source affecting all objects in the scene equally, set as a resource
pub struct AmbientLight {
    pub color: Color,
    pub intensity: f32,
}

/// Light source emitting light orthogonally in a specific direction with a orthographic projection
/// (sunlight)
/// Direction is extracted from the transform component
pub struct DirectionalLight {
    pub color: Color,
    pub intensity: f32,
    pub shadow: bool,
}

/// Light source emitting light in all directions from a point in space (light bulb)
/// Position is extracted from the transform component
pub struct PointLight {
    pub color: Color,
    pub intensity: f32,
    pub shadow: bool,
    pub range: f32,
}

/// Light source emitting cone light in a specific direction with a perspective projection
/// (flashlight or car headlight)
/// Position and direction are extracted from the transform component
pub struct SpotLight {
    pub color: Color,
    pub intensity: f32,
    pub shadow: bool,
    pub range: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            color: palette::WHITE.as_rgba_slice(),
            intensity: 1.0,
            flags: LightFlags::Visible | LightFlags::Ambient,  
            range: 0.0,
            inner_angle: 0.0,
            outer_angle: 0.0,
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),

            _padding: [0.0; 3],
        }
    }
}

impl AmbientLight {
    pub fn as_light(&self, view_projection_matrix: Mat4) -> Light {
        Light {
            color: self.color.as_rgba_slice(),
            intensity: self.intensity,
            flags: LightFlags::Visible | LightFlags::Ambient,
            view_proj: view_projection_matrix.to_cols_array_2d(),
            ..Default::default()
        }
    }
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: palette::WHITE,
            intensity: 1.0,
        }
    }
}

impl DirectionalLight {
    pub fn as_light(&self, view_projection_matrix: Mat4) -> Light {
        let mut flags = LightFlags::Visible | LightFlags::Directional;
        if self.shadow {
            flags |= LightFlags::CastShadow as u32 
        }
        
        Light {
            color: self.color.as_rgba_slice(),
            intensity: self.intensity,
            flags,
            view_proj: view_projection_matrix.to_cols_array_2d(),
            ..Default::default()
        }
    }

    pub fn view_matrix(&self, camera_position: Vec3, rotation: Quat) -> Mat4 {
        // Local space light direction (-Y) and up vector (-Z)
        let local_direction = Vec3::new(0.0, -1.0, 0.0);
        let local_up = Vec3::new(0.0, 0.0, -1.0);

        // Rotate the direction and up vectors by the light's rotation
        let world_direction = rotation * local_direction;
        let world_up = rotation * local_up;

        // Offset camera's position by the direction to track the camera
        let light_position = camera_position - world_direction * 10.0;

        Mat4::look_at_rh(light_position, camera_position, world_up)
    }

    pub fn projection_matrix(&self, size: f32, near_plane: f32, far_plane: f32) -> Mat4 {
        Mat4::orthographic_rh(-size, size, -size, size, near_plane, far_plane)
    }

    pub fn view_projection_matrix(&self, size: f32, near_plane: f32, far_plane: f32, camera_position: Vec3, global_transform: Mat4) -> Mat4 {
        // Extract the rotation from the global transform
        let rotation = global_transform.to_scale_rotation_translation().1;

        self.projection_matrix(size, near_plane, far_plane) * self.view_matrix(camera_position, rotation)
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            color: palette::WHITE,
            intensity: 1.0,
            shadow: true,
        }
    }
}

impl PointLight {
    pub fn as_light(&self, view_projection_matrix: Mat4) -> Light {
        let mut flags = LightFlags::Visible | LightFlags::Point;
        if self.shadow { 
            flags |= LightFlags::CastShadow as u32 
        }
        
        Light {
            color: self.color.as_rgba_slice(),
            intensity: self.intensity,
            flags,
            range: self.range,
            view_proj: view_projection_matrix.to_cols_array_2d(),
            ..Default::default()
        }
    }

    pub fn view_matrix_for_face(&self, position: Vec3, face: CubeMapFace) -> Mat4 {
        // Look direction for each cube map face.
        let (eye, direction, up) = match face {
            CubeMapFace::PosX => (position, position + Vec3::X, Vec3::Y),
            CubeMapFace::NegX => (position, position - Vec3::X, Vec3::Y),
            CubeMapFace::PosY => (position, position + Vec3::Y, Vec3::NEG_Z),
            CubeMapFace::NegY => (position, position - Vec3::Y, Vec3::Z),
            CubeMapFace::PosZ => (position, position + Vec3::Z, Vec3::Y),
            CubeMapFace::NegZ => (position, position - Vec3::Z, Vec3::Y),
        };

        Mat4::look_at_rh(eye, direction, up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        let fov = std::f32::consts::PI / 2.0; // 90 degrees in radians.
        let aspect = 1.0; // Aspect ratio is 1:1 for cube maps

        Mat4::perspective_rh(fov, aspect, 0.1, self.range)
    }

    pub fn view_proj_matrix_for_face(&self, position: Vec3, face: CubeMapFace) -> Mat4 {
        self.projection_matrix() * self.view_matrix_for_face(position, face)
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: palette::WHITE,
            intensity: 1.0,
            shadow: true,
            range: 10.0,
        }
    }
}

impl SpotLight {
    pub fn as_light(&self, view_projection_matrix: Mat4) -> Light {
        let mut flags = LightFlags::Visible | LightFlags::Spot;
        if self.shadow { 
            flags |= LightFlags::CastShadow as u32 
        }

        Light {
            color: self.color.as_rgba_slice(),
            intensity: self.intensity,
            flags,
            range: self.range,
            inner_angle: self.inner_angle,
            outer_angle: self.outer_angle,
            view_proj: view_projection_matrix.to_cols_array_2d(),
            ..Default::default()
        }
    }

    pub fn view_matrix(&self, position: Vec3, rotation: Quat) -> Mat4 {
        // Local space light direction (-Y) and up vector (-Z)
        let local_direction = Vec3::new(0.0, -1.0, 0.0);
        let local_up = Vec3::new(0.0, 0.0, -1.0);

        // Rotate the direction and up vectors by the light's rotation
        let world_direction = rotation * local_direction;
        let world_up = rotation * local_up;

        Mat4::look_at_rh(position, position + world_direction, world_up)
    }

    pub fn projection_matrix(&self, aspect: f32, near_plane: f32) -> Mat4 {
        Mat4::perspective_rh(
            self.outer_angle.to_radians() * 2.0, 
            aspect, 
            near_plane, 
            self.range
        )
    }

    pub fn view_projection_matrix(&self, aspect: f32, near_plane: f32, global_transform: Mat4) -> Mat4 {
        // Extract the position and rotation from the global transform
        let (_, rotation, position) = global_transform.to_scale_rotation_translation();

        self.projection_matrix(aspect, near_plane) * self.view_matrix(position, rotation)
    }
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            color: palette::WHITE,
            intensity: 1.0,
            shadow: true,
            range: 50.0,
            inner_angle: 15.0,
            outer_angle: 45.0,
        }
    }
}
