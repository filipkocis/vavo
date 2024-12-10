use crate::{assets::Handle, render_assets::{BindGroup, Buffer, RenderAsset, RenderAssets}, renderer::{palette, Color, Image}};

use super::{Rect, Transform};

/// Main camera component
/// Requires Projection, Transform, and Camera2D/3D components
pub struct Camera {
    pub active: bool,
    pub target: Option<Handle<Image>>,
    pub clear_color: Color,
}

/// Defines a 3D camera, required for 3D rendering
pub struct Camera3D {}

/// Projection type component, required for camera
pub enum Projection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection),
}

/// Used in Projection enum for camera
pub struct PerspectiveProjection {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

/// Used in Projection enum for camera
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
    pub fn get_view_projection_matrix(&self, transform: &Transform) -> [[f32; 4]; 4] {
        match self {
            Projection::Perspective(p) => {
                let projection = glam::Mat4::perspective_rh(p.fov.to_radians(), p.aspect_ratio, p.near, p.far);
                let view = transform.as_matrix().inverse();
                let view_projection = projection * view;
                
                view_projection.to_cols_array_2d()
            },
            Projection::Orthographic(o) => {
                let projection = glam::Mat4::orthographic_rh(
                    o.area.min.x, o.area.max.x, o.area.min.y, o.area.max.y, o.near, o.far
                );
                let view = transform.as_matrix().inverse();
                let view_projection = projection * view;
                
                view_projection.to_cols_array_2d()
            }
        }
    }
}

impl Camera {
    pub fn get_buffer_data(projection: &Projection, transform: &Transform) -> [[f32; 4]; 4] {
        projection.get_view_projection_matrix(transform)
    }
}

impl RenderAsset<Buffer> for Camera {
    fn create_render_asset(
            &self, 
            ctx: &mut crate::prelude::SystemsContext,
            _: Option<&crate::prelude::EntityId>
    ) -> Buffer {
        // TODO: implement some query system for components, for now we use defaults
        let transform = crate::math::Transform::default()
            .with_translation(glam::Vec3::new(0.0, 0.0, 10.0));
        let projection = Projection::perspective();

        let data = match projection {
            Projection::Perspective(p) => {
                let projection = glam::Mat4::perspective_rh(p.fov.to_radians(), p.aspect_ratio, p.near, p.far);
                let view = transform.as_matrix().inverse();
                let view_projection = projection * view;
                
                view_projection.to_cols_array_2d()
            },
            Projection::Orthographic(o) => {
                let projection = glam::Mat4::orthographic_rh(
                    o.area.min.x, o.area.max.x, o.area.min.y, o.area.max.y, o.near, o.far
                );
                let view = transform.as_matrix().inverse();
                let view_projection = projection * view;
                
                view_projection.to_cols_array_2d()
            }
        };
        
        Buffer::new("camera")
            .create_uniform_buffer(&[data], Some(wgpu::BufferUsages::COPY_DST), ctx.renderer.device())
    }
}

impl RenderAsset<BindGroup> for Camera {
    fn create_render_asset(
            &self, 
            ctx: &mut crate::prelude::SystemsContext,
            entity_id: Option<&crate::prelude::EntityId>
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
