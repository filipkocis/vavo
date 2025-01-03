use glyphon::{TextAtlas, TextRenderer, Viewport};

use crate::prelude::*;
use crate::core::graph::*;

use crate::render_assets::{BindGroup, Buffer, RenderAssets};
use crate::ui::mesh::UiMesh;
use crate::ui::node::{ComputedNode, Node};

use super::UiTransformStorage;

/// Ui graph node rendering system
pub fn ui_render_system(
    grpah_ctx: RenderGraphContext,
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

    let render_pass = grpah_ctx.pass;

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
    // text_atlas.trim();
}

// /// System to update the UI mesh and UI transform storage
// pub fn update_ui_mesh_and_transforms<'a>(
//     ctx: &mut SystemsContext,
//     mut query: Query<(&'a EntityId, &'a Transform, &'a GlobalTransform, &'a Node, &'a ComputedNode), Changed<Transform>>,
// ) {
//     let mut ui_transform_storage = ctx.resources.get_mut::<UiTransformStorage>().expect("UiTransformStorage resource not found");
//     let mut ui_mesh = ctx.resources.get_mut::<UiMesh>().expect("UiMesh resource not found");
//     let mut ui_transforms = Vec::new();
//     let mut transform_index = 0;
//     let size = *ctx.renderer.size();
//
//     let time = ctx.resources.get::<Time>().expect("Time not oufnd");
//     // if time.tick() > 1 {
//         // return;
//     // }
//     ui_mesh.clear();
//     // println!("running update ui mesh transform system");
//
//     let ui_nodes = query.iter_mut();
//
//     let mut text_query = query.cast::<&Text, With<Node>>();
//     let mut text_buffers = ctx.resources.get_mut::<RenderAssets<TextBuffer>>().expect("TextBuffer render assets not found");
//     let mut font_system = ctx.resources.get_mut::<FontSystem>().expect("FontSystem resource not found");
//     let mut swash_cache = ctx.resources.get_mut::<SwashCache>().expect("SwashCache resource not found");
//
//     // add other nodes as options 
//     // TODO: implement Option<T> to query
//     let ui_nodes = ui_nodes.into_iter().map(|(id, transform, global_transform, node, computed)| {
//         let text = if let Some(text) = text_query.get(*id) {
//             let text = text_buffers.get_by_entity(id, text, ctx);
//             Some(text)
//         } else {
//             None
//         };
//         
//         (id, transform, global_transform, node, computed, text)
//     });
//
//     for (id, transform, global_transform, node, computed, text) in ui_nodes {
//         // println!("updating {:?}", id);
//         // node
//         if node.background_color != palette::TRANSPARENT {
//             ui_mesh.add_rect(
//                 transform.translation.x,
//                 size.height as f32 - transform.translation.y,
//                 computed.width,
//                 -computed.height,
//                 node.background_color,
//                 transform_index,
//             );
//         }
//
//         // text
//         if let Some(text) = text {
//             // println!("drawing text ui node");
//             // TODO: computed.color
//             text.buffer.draw(&mut font_system, &mut swash_cache, node.color.into(), |x, y, w, h, color| {
//                 ui_mesh.add_rect(
//                     x as f32,
//                     size.height as f32 - y as f32,
//                     w as f32,
//                     -(h as f32),
//                     color.into(),
//                     transform_index,
//                 );
//             });
//         } 
//
//         // entitie's transform
//         ui_transforms.push(global_transform.as_matrix().to_cols_array_2d());
//
//         transform_index += 1;
//     }
//
//     // dbg!(&*ui_mesh.positions);
//     // panic!();
//
//     ui_transform_storage.update(&ui_transforms, ui_transforms.len(), ctx);
// }

/// System to initialize new UI nodes, it adds Transform and ComputedNode components
pub fn initialize_ui_nodes(
    ctx: &mut SystemsContext,
    mut query: Query<&EntityId, (With<Node>, Without<Transform>, Without<ComputedNode>)>,
) {
    for id in query.iter_mut() {
        ctx.commands.entity(*id)
            .insert(Transform::default())
            .insert(ComputedNode::default());

        println!("Initialized ui node: {:?}", id);
    }
}

// /// Updates the computed node and transform components of all UI nodes based on their parents
// pub fn compute_nodes_and_transforms(ctx: &mut SystemsContext, mut q: Query<()>) {
//     let mut query = q.cast::<(&EntityId, &Node, &mut ComputedNode, &mut Transform), Changed<Node>>();
//     let mut ui_nodes = Vec::new();
//     for node in query.iter_mut() {
//         ui_nodes.push(node)
//     }
//
//     // println!("running compute nodes and transforms");
//     let mut compute_context = ComputeContext::from_size(*ctx.renderer.size());
//
//     let mut has_parent = q.cast::<&Parent, ()>();
//     let parents = ui_nodes.into_iter().filter_map(|(id, node, computed, transform)| {
//         match has_parent.get(*id) {
//             //
//             // TODO: THIS IS WRONG, swap order and update child if parent was not updated
//             //
//             Some(parent) => Some((*id, parent, node, computed, transform)),
//             None => {
//                 // if no parent, update just this node
//                 *computed = node.compute(&compute_context, ctx);
//                 transform.translation = computed.self_translation();
//
//                 return None
//             },
//         }
//     }).collect::<Vec<_>>();
//
//     // let mut is_parent_ui = q.cast::<&Node, ()>();
//     for (id, parent, node, computed, transform) in parents {
//         // if is_parent_ui.get(parent.id).is_none(){
//         //     continue;
//         // }
//
//         compute_context.set_parent(None);
//         *computed = node.compute(&compute_context, ctx); 
//         compute_context.set_parent(Some((node, computed)));
//
//         transform.translation = computed.self_translation();
//
//         update_children(id, &mut compute_context, q.cast(), ctx);
//     }
//
//     super::compute::compute_nodes_and_transforms(ctx, q);
// }
//
// fn update_children<'a>(
//     parent_id: EntityId, 
//     compute_context: &mut ComputeContext<'a>, 
//     mut parent_query: Query<&Children>, 
//     ctx: &mut SystemsContext
// ) {
//     // get children of parent
//     let children = match parent_query.get(parent_id) {
//         Some(children) => children,
//         None => return,
//     };
//
//     // update every child recursively
//     let mut child_query = parent_query.cast::<(&Node, &mut ComputedNode, &mut Transform), With<Parent>>();
//     let mut accumulated_translation = Vec3::ZERO;
//     for (_, child) in children.ids.iter().enumerate() {
//         if let Some((node, computed, transform)) = child_query.get(*child) {
//             // update child of parent
//             *computed = node.compute(compute_context, ctx);
//             transform.translation = computed
//                 .children_translation(
//                     compute_context.parent_computed.expect("Parent computed node not set"),
//                     &accumulated_translation,
//                 );
//             compute_context.set_parent(Some((node, computed)));
//
//             // TODO: handle grid flex layout
//             accumulated_translation.x += computed.width + computed.column_gap;
//             // accumulated_translation.y += computed.height + computed.row_gap;
//
//             // recursively update children of child
//             update_children(*child, compute_context, child_query.cast(), ctx);
//         } 
//     }
// }
