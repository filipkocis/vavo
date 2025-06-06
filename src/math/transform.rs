use glam::{Mat4, Quat, Vec3, Vec4Swizzles};

use crate::{render_assets::{BindGroup, Buffer, IntoRenderAsset, RenderAssets}, system::SystemsContext, ecs::entities::EntityId, macros::{Component, Reflect}};

/// Represents the local transform of an entity, relative to its parent or the world space if it
/// has no parent.
#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct Transform {
    pub scale: Vec3,
    pub rotation: Quat,
    pub translation: Vec3,
}

/// GlobalTransform represents the world-space transform of an entity.
/// If an entity has a parent, it will be calculated as the parent's GlobalTransform * child's
/// local Transform.
///
/// # Note
/// This component is added automatically when a Transform component is added to an entity.
#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct GlobalTransform {
    pub matrix: Mat4,
}

impl GlobalTransform {
    pub fn new(matrix: Mat4) -> Self {
        Self { matrix }
    }

    /// Extract the translation component
    pub fn translation(&self) -> Vec3 {
        self.matrix.w_axis.xyz()
    }

    /// Extract the rotation component
    pub fn rotation(&self) -> Quat {
        self.matrix.to_scale_rotation_translation().1            
    }

    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            matrix: transform.as_matrix()
        }
    }

    /// Update the global transform based on the provided local transform.
    pub fn update(&mut self, local: &Transform) {
        self.matrix = local.as_matrix();
    }

    pub fn as_matrix(&self) -> Mat4 {
        self.matrix
    }

    /// Combine this global transform with a child local transform, returning a new global
    /// transform for the child.
    pub fn combine_child(&self, child_local: &Transform) -> Self {
        Self {
            matrix: self.matrix * child_local.as_matrix()
        }
    }
}

impl Transform {
    /// Create new default Transform
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new Transform from a matrix
    pub fn from_matrix(matrix: &Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();

        Self {
            scale, rotation, translation
        }
    }

    /// Returns new 3D transformation matrix
    pub fn as_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
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

impl IntoRenderAsset<Buffer> for Transform {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        _: Option<EntityId>
    ) -> Buffer {
        let data = self.as_matrix().to_cols_array_2d();

        Buffer::new("transform")
            .create_uniform_buffer(&[data], Some(wgpu::BufferUsages::COPY_DST), ctx.renderer.device())
    }
}   

impl IntoRenderAsset<BindGroup> for Transform {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        entity_id: Option<EntityId>
    ) -> BindGroup {
        let id = entity_id.expect("EntityId should be provided for Transform BindGroup");

        let mut buffers = ctx.resources.get_mut::<RenderAssets<Buffer>>().unwrap();
        let buffer = buffers.get_by_entity(id, self, ctx);
        let uniform_buffer = buffer.uniform.as_ref().expect("Transform buffer should be uniform");

        BindGroup::build("transform")
            .add_uniform_buffer(uniform_buffer, wgpu::ShaderStages::VERTEX)
            .finish(ctx)
    }
}
