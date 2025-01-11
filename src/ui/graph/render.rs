use glyphon::{TextAtlas, TextRenderer, Viewport};

use crate::prelude::*;
use crate::core::graph::*;

use crate::render_assets::{BindGroup, Buffer, RenderAssets};
use crate::ui::mesh::UiMesh;

use super::UiTransformStorage;

/// Ui graph node rendering system
pub fn ui_render_system(
    graph_ctx: RenderGraphContext,
    ctx: &mut SystemsContext,
    mut query: Query<()>,
) {
    // resources
    let mut buffers = ctx.resources.get_mut::<RenderAssets<Buffer>>().unwrap();
    let mut bind_groups = ctx.resources.get_mut::<RenderAssets<BindGroup>>().unwrap();
    // text resources
    let text_renderer = ctx.resources.get::<TextRenderer>().expect("TextRenderer resource not found");
    let text_atlas = ctx.resources.get::<TextAtlas>().expect("TextAtlas resource not found");
    let viewport = ctx.resources.get::<Viewport>().expect("Viewport resource not found");

    // holds the ui mesh - vertices and indices for every ui node
    let ui_mesh = ctx.resources.get::<UiMesh>().expect("UiMesh resource not found");  
    let ui_mesh = buffers.get_by_resource(&ui_mesh, ctx, true);

    if ui_mesh.num_vertices == 0 {
        return;
    }

    // holds the transform of every ui node
    let ui_transforms = ctx.resources.get::<UiTransformStorage>().expect("UiTransformStorage resource not found");

    let vertex_buffer = ui_mesh.vertex.as_ref().expect("UiMesh buffer should be vertex buffer");
    let index_buffer =  ui_mesh.index.as_ref().expect("UiMesh buffer should be index buffer");

    let mut camera_query = query.cast::<(&EntityId, &Camera), (With<Transform>, With<Projection>, With<Camera3D>)>(); 
    let active_camera = camera_query.iter_mut().into_iter().filter(|(_, c)| c.active).take(1).next();
    let camera_bind_group;
    if let Some((id, camera)) = active_camera {
        camera_bind_group = bind_groups.get_by_entity(id, camera, ctx);
    } else {
        return;
    }

    let render_pass = graph_ctx.pass;

    // window size
    render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::cast_slice(&[
        (ctx.renderer.size().width as f32),
        (ctx.renderer.size().height as f32)
    ]));

    render_pass.set_bind_group(0, &*ui_transforms.bind_group(), &[]);
    render_pass.set_bind_group(1, &*camera_bind_group, &[]);

    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    render_pass.draw_indexed(0..ui_mesh.num_indices, 0, 0..1);

    // render text
    text_renderer.render(&text_atlas, &viewport, render_pass).unwrap();
}
