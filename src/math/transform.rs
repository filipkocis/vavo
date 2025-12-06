use glam::{Mat3, Mat4, Quat, Vec3, Vec4Swizzles};

use crate::{
    ecs::entities::EntityId,
    macros::{Component, Reflect},
    prelude::World,
    render_assets::{BindGroup, Buffer, IntoRenderAsset, RenderAssets},
};

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
            matrix: transform.as_matrix(),
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
            matrix: self.matrix * child_local.as_matrix(),
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
            scale,
            rotation,
            translation,
        }
    }

    /// Returns new 3D transformation matrix
    #[inline]
    pub fn as_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    #[inline]
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    #[inline]
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    #[inline]
    pub fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    /// Creates a new Transform with its rotation pointing in the direction of `target`,
    /// using `up` as the up vector.
    #[inline]
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        self.look_at(target, up);
        self
    }

    /// Creates a new Transform with its rotation pointing in the `direction`, using `up` as the up
    /// vector.
    #[inline]
    pub fn looking_to(mut self, direction: Vec3, up: Vec3) -> Self {
        self.look_to(direction, up);
        self
    }

    #[inline]
    pub fn translate(&mut self, delta: Vec3) {
        self.translation += delta;
    }

    #[inline]
    pub fn rotate(&mut self, delta: Quat) {
        // Note: order matters here
        self.rotation = delta * self.rotation;
    }

    #[inline]
    pub fn scale(&mut self, delta: Vec3) {
        self.scale *= delta;
    }

    /// Rotates this transform to look in the direction of `target` with the `up` vector
    #[inline]
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let direction = target - self.translation;
        self.look_to(direction, up);
    }

    /// Rotates this transform to look in the `direction` with the `up` vector
    #[inline]
    pub fn look_to(&mut self, direction: Vec3, up: Vec3) {
        let back = -direction.normalize_or(Vec3::NEG_Z);
        let up = up.normalize_or(Vec3::Y);

        let right = up
            .cross(back)
            .try_normalize()
            .unwrap_or_else(|| up.any_orthonormal_vector());
        let up = back.cross(right);

        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, back));
    }

    /// Get the forward direction vector (negative Z axis)
    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::NEG_Z)
    }

    /// Get the up direction vector (Y axis)
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::Y)
    }

    /// Get the right direction vector (X axis)
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::X)
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
    fn create_render_asset(&self, world: &mut World, _: Option<EntityId>) -> Buffer {
        let data = self.as_matrix().to_cols_array_2d();

        Buffer::new("transform").create_uniform_buffer(
            &[data],
            Some(wgpu::BufferUsages::COPY_DST),
            &world.resources.get(),
        )
    }
}

impl IntoRenderAsset<BindGroup> for Transform {
    fn create_render_asset(&self, world: &mut World, entity_id: Option<EntityId>) -> BindGroup {
        let id = entity_id.expect("EntityId should be provided for Transform BindGroup");

        let mut buffers = world.resources.get_mut::<RenderAssets<Buffer>>();
        let buffer = buffers.get_by_entity(id, self, world);
        let uniform_buffer = buffer
            .uniform
            .as_ref()
            .expect("Transform buffer should be uniform");

        BindGroup::build("transform")
            .add_uniform_buffer(uniform_buffer, wgpu::ShaderStages::VERTEX)
            .finish(&world.resources.get())
    }
}
