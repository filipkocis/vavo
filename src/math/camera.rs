use glam::Mat4;

use crate::{assets::Handle, render_assets::{BindGroup, Buffer, IntoRenderAsset, RenderAssets}, renderer::{palette, Color, Image}, system::SystemsContext, ecs::entities::EntityId, macros::{Component, Reflect}};

use super::{GlobalTransform, Rect};

/// Main camera component
/// Requires Projection, Transform, and Camera2D/3D components
#[derive(Component)]
pub struct Camera {
    pub active: bool,
    pub target: Option<Handle<Image>>,
    pub clear_color: Color,
}

/// Defines a 3D camera, required for 3D rendering
#[derive(Component, Reflect)]
pub struct Camera3D {}

/// Projection type component, required for camera
#[derive(Component, Reflect)]
pub enum Projection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection),
}

/// Used in Projection enum for camera
#[derive(Component, Reflect)]
pub struct PerspectiveProjection {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

/// Used in Projection enum for camera
#[derive(Component, Reflect)]
pub struct OrthographicProjection {
    pub area: Rect,
    pub scale: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            active: true,
            target: None,
            clear_color: palette::BLACK,
        }
    }
}

impl Default for Camera3D {
    fn default() -> Self {
        Self {}
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            fov: 45.0,
            near: 0.1,
            far: 100.0,
            aspect_ratio: 1.0,
        }
    }
} 

impl Default for OrthographicProjection {
    fn default() -> Self {
        Self {
            area: Rect::new_min_max(-400., -300., 400., 300.),
            scale: 1.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

impl Projection {
    pub fn perspective() -> Self {
        Self::Perspective(PerspectiveProjection::default())
    }

    pub fn orthographic() -> Self {
        Self::Orthographic(OrthographicProjection::default())
    }
}

impl Projection {
    pub fn get_view_projection_matrix(&self, matrix: &Mat4) -> [[f32; 4]; 4] {
        let view = matrix.inverse();

        match self {
            Projection::Perspective(p) => {
                let projection = glam::Mat4::perspective_rh(p.fov.to_radians(), p.aspect_ratio, p.near, p.far);
                let view_projection = projection * view;
                
                view_projection.to_cols_array_2d()
            },
            Projection::Orthographic(o) => {
                let projection = glam::Mat4::orthographic_rh(
                    o.area.min.x, o.area.max.x, o.area.min.y, o.area.max.y, o.near, o.far
                );
                let view_projection = projection * view;
                
                view_projection.to_cols_array_2d()
            }
        }
    }

    /// Resize the projection `aspect ratio` / `area` based on new width and height
    pub fn resize(&mut self, width: f32, height: f32) {
        match self {
            Projection::Perspective(p) => {
                p.aspect_ratio = width / height;
            },
            Projection::Orthographic(o) => {
                o.area = Rect::new_min_max(-width / 2.0, -height / 2.0, width / 2.0, height / 2.0);
            }
        }
    }
}

impl Camera {
    pub fn get_buffer_data(projection: &Projection, global_transform: &GlobalTransform) -> Vec<f32> {
        let mut data = projection.get_view_projection_matrix(&global_transform.matrix).as_flattened().to_vec();
        let translation = global_transform.translation();

        data.extend(&[
            translation.x,
            translation.y,
            translation.z,
            0.0, // padding
        ]);
        data
    }
}

impl IntoRenderAsset<Buffer> for Camera {
    fn create_render_asset(
            &self, 
            ctx: &mut SystemsContext,
            entity_id: Option<EntityId>
    ) -> Buffer {
        let id = entity_id.expect("EntityId should be provided for Camera Buffer");

        let world = &unsafe { &mut *ctx.app }.world;
        let projection = world.entities.get_component(id).expect("Camera should have a Projection component");
        let global_transform = world.entities.get_component(id).expect("Camera should have a GlobalTransform component");

        let data = Camera::get_buffer_data(projection, global_transform);
        
        Buffer::new("camera")
            .create_uniform_buffer(&data, Some(wgpu::BufferUsages::COPY_DST), ctx.renderer.device())
    }
}

impl IntoRenderAsset<BindGroup> for Camera {
    fn create_render_asset(
            &self, 
            ctx: &mut SystemsContext,
            entity_id: Option<EntityId>
    ) -> BindGroup {
        let id = entity_id.expect("EntityId should be provided for Camera BindGroup");

        let mut buffers = ctx.resources.get_mut::<RenderAssets<Buffer>>().unwrap();
        let buffer = buffers.get_by_entity(id, self, ctx); 
        let uniform_buffer = buffer.uniform.as_ref().expect("Camera buffer should be uniform");

        BindGroup::build("camera")
            .add_uniform_buffer(uniform_buffer, wgpu::ShaderStages::VERTEX_FRAGMENT)
            .finish(ctx)
    }
}
