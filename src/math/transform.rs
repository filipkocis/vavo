use glam::{Quat, Vec3};

use crate::render_assets::{Buffer, RenderAsset};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub scale: Vec3,
    pub rotation: Quat,
    pub translation: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    pub fn as_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            translation: Vec3::ZERO,
        }
    }
}

impl RenderAsset<Buffer> for Transform {
    fn create_render_asset(&self, device: &wgpu::Device, _: &mut crate::prelude::Resources) -> Buffer {
        let data = self.as_matrix().to_cols_array_2d();

        Buffer::new("transform")
            .create_uniform_buffer(&[data], Some(wgpu::BufferUsages::COPY_DST), device)
    }
}
