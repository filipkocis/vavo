use glyphon::{TextAtlas, TextRenderer, Viewport};
use winit::dpi::PhysicalSize;

use crate::prelude::*;
use crate::core::graph::*;

use crate::render_assets::{BindGroup, Buffer, RenderAssets};
use crate::ui::mesh::{UiMesh, UiMeshTransparent};

use super::storage::UiTransformStorage;

/// Ui graph node rendering system
pub fn ui_render_system(
    graph_ctx: CustomRenderGraphContext,
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
    let ui_mesh_transparent = ctx.resources.get::<UiMeshTransparent>().expect("UiMeshTransparent resource not found");  
    let ui_mesh = buffers.get_by_resource(&ui_mesh, ctx, true);
    let ui_mesh_transparent = buffers.get_by_resource(&ui_mesh_transparent, ctx, true);

    // holds the transform of every ui node
    let ui_transforms = ctx.resources.get::<UiTransformStorage>().expect("UiTransformStorage resource not found");

    let mut camera_query = query.cast::<(&EntityId, &Camera), (With<Transform>, With<Projection>, With<Camera3D>)>(); 
    let active_camera = camera_query.iter_mut().into_iter().filter(|(_, c)| c.active).take(1).next();
    let camera_bind_group;
    if let Some((id, camera)) = active_camera {
        camera_bind_group = bind_groups.get_by_entity(id, camera, ctx);
    } else {
        return;
    }

    // prepare attachments
    let color_attachment = Some(wgpu::RenderPassColorAttachment {
        view: unsafe { &*graph_ctx.color_target.expect("ui color target is None") },
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
        }
    });

    let mut depth_stencil = wgpu::RenderPassDepthStencilAttachment {
        view: unsafe { &*graph_ctx.depth_target.expect("ui depth target is None") },
        depth_ops: Some(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
        }),
        stencil_ops: None,
    };

    let pipeline = unsafe { &*graph_ctx.node }.data.pipeline.as_ref().expect("Pipeline should have been generated by now").render_pipeline();
    let mut encoder = ctx.renderer.encoder();

    // opaque render pass
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ui opaque render pass"),
            color_attachments: &[color_attachment.clone()],
            depth_stencil_attachment: Some(depth_stencil.clone()), 
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        draw_ui_render_pass(&mut render_pass, &pipeline, ctx.renderer.size(), ui_transforms.bind_group(), &*camera_bind_group, &*ui_mesh);
    } // necessary to drop render_pass before second pass

    // dont store depth for transparent objects
    depth_stencil.depth_ops = Some(wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Discard,
    });

    // transparent render pass
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ui transparent render pass"),
        color_attachments: &[color_attachment],
        depth_stencil_attachment: Some(depth_stencil), 
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    draw_ui_render_pass(&mut render_pass, &pipeline, ctx.renderer.size(), ui_transforms.bind_group(), &*camera_bind_group, &*ui_mesh_transparent);

    // render text
    text_renderer.render(&text_atlas, &viewport, &mut render_pass).unwrap();
}

fn draw_ui_render_pass(
    render_pass: &mut wgpu::RenderPass,
    pipeline: &wgpu::RenderPipeline,
    window_size: PhysicalSize<u32>,
    ui_transforms_bind_group: &wgpu::BindGroup,
    camera_bind_group: &BindGroup,
    ui_mesh: &Buffer,
) {
    if ui_mesh.num_indices == 0 {
        return;
    }

    let vertex_buffer = ui_mesh.vertex.as_ref().expect("UiMesh buffer should have a vertex buffer");
    let index_buffer = ui_mesh.index.as_ref().expect("UiMesh buffer should have an index buffer");

    render_pass.set_pipeline(pipeline);

    // push constants
    render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::cast_slice(&[
        (window_size.width as f32),
        (window_size.height as f32)
    ]));

    // bind groups
    render_pass.set_bind_group(0, ui_transforms_bind_group, &[]);
    render_pass.set_bind_group(1, camera_bind_group, &[]);

    // vertex and index buffers
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    
    // draw
    render_pass.draw_indexed(0..ui_mesh.num_indices, 0, 0..1);
}
