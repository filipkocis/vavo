use crate::core::graph::*;
use crate::prelude::*;
use crate::render_assets::{BindGroup, Buffer, RenderAssets};
use crate::ui::{graph::storage::UiTransformStorage, mesh::UiMeshImages, prelude::*};

pub fn ui_image_render_system(
    graph_ctx: Res<RenderContext>,

    world: &mut World,
    window: Res<Window>,

    // resources
    mut buffers: ResMut<RenderAssets<Buffer>>,
    mut bind_groups: ResMut<RenderAssets<BindGroup>>,
    ui_mesh_images: Res<UiMeshImages>,

    // holds the transform of every ui node
    ui_transforms: Res<UiTransformStorage>,

    mut camera_query: Query<
        (EntityId, &Camera),
        (With<Transform>, With<Projection>, With<Camera3D>),
    >,
    mut ui_image_query: Query<&UiImage, With<Node>>,
) {
    let ui_mesh_images_buffer = buffers.get_by_resource(&ui_mesh_images, world, true);
    if ui_mesh_images_buffer.num_vertices == 0 {
        return;
    }

    // find active camera
    let active_camera = camera_query
        .iter_mut()
        .into_iter()
        .filter(|(_, c)| c.active)
        .take(1)
        .next();
    let camera_bind_group;
    if let Some((id, camera)) = active_camera {
        camera_bind_group = bind_groups.get_by_entity(id, camera, world);
    } else {
        return;
    }

    // extract buffers
    let vertex_buffer = ui_mesh_images_buffer
        .vertex
        .as_ref()
        .expect("UiMeshImages buffer should have a vertex buffer");
    let index_buffer = ui_mesh_images_buffer
        .index
        .as_ref()
        .expect("UiMeshImages buffer should have an index buffer");

    let render_pass = unsafe { &mut *graph_ctx.pass };

    // bind groups
    render_pass.set_bind_group(0, ui_transforms.bind_group(), &[]);
    render_pass.set_bind_group(1, &*camera_bind_group, &[]);

    // vertex and index buffers
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

    // push constants
    let window_size = window.size();
    render_pass.set_push_constants(
        wgpu::ShaderStages::VERTEX,
        0,
        bytemuck::cast_slice(&[(window_size.width as f32), (window_size.height as f32)]),
    );

    // loop through all ui nodes
    let mut current_indices = 0..6;
    for &entity_id in &ui_mesh_images.entity_ids {
        // get image
        let image = ui_image_query
            .get(entity_id)
            .expect("UiImage component not found");
        let image_bind_group = bind_groups.get_by_entity(entity_id, image, world);

        // per entity bind group
        render_pass.set_bind_group(2, &*image_bind_group, &[]);

        // draw
        render_pass.draw_indexed(current_indices.clone(), 0, 0..1);

        // move to next rect
        current_indices.start = current_indices.end;
        current_indices.end = current_indices.start + 6;
    }
}
