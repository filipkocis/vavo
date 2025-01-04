use glam::Vec2;
use glyphon::{FontSystem, Resolution, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};

use crate::{palette, prelude::*};
use crate::render_assets::RenderAssets;
use crate::ui::mesh::UiMesh;
use crate::ui::node::{ComputedNode, Display, Node};
use crate::ui::text::text::{Text, TextBuffer};

use super::UiTransformStorage;

/// System to update the glyphon text viewport resolution. 
/// Runs only if the window size has changed.
pub fn update_glyphon_viewport(ctx: &mut SystemsContext, _: Query<()>) {
    let mut viewport = ctx.resources.get_mut::<Viewport>().expect("Viewport resource not found");
    let queue = ctx.renderer.queue();
    let size = ctx.renderer.size();

    viewport.update(queue, Resolution {
        width: size.width,
        height: size.height,
    })
}

/// System to update the UI mesh and UI transform storage, runs only if some nodes have `Changed<Transform>` filter.
/// The filter should return true after `compute_node` system which runs on `Changed<Node>` filter and updates transforms.
///
/// # Note
/// Applies z-index to the z component of the global transfrom pushed to the transform storage.
pub fn update_ui_mesh_and_transforms(ctx: &mut SystemsContext, mut query: Query<()>) {
    // resources
    let mut ui_transform_storage = ctx.resources.get_mut::<UiTransformStorage>().expect("UiTransformStorage resource not found");
    let mut ui_mesh = ctx.resources.get_mut::<UiMesh>().expect("UiMesh resource not found");

    // get the amount of changed nodes
    let mut changed_query = query.cast::<&EntityId, (
        With<Transform>, With<GlobalTransform>, With<Node>, With<ComputedNode>, Changed<Transform>
    )>();
    let changed_len = changed_query.iter_mut().len();

    // query all nodes
    let mut nodes_query = query.cast::<(&EntityId, &Transform, &GlobalTransform, &Node, &ComputedNode), ()>();
    let ui_nodes = nodes_query.iter_mut();

    // return if nothing changed
    if changed_len == 0 {
        // cleanup if all nodes were removed
        if ui_nodes.len() == 0 && ui_mesh.positions.len() > 0 {
            ui_mesh.clear();
        }
        return;
    }

    // clear the mesh and regenerate all nodes
    // TODO: optimize this
    ui_mesh.clear();

    // text resources
    let mut text_buffers = ctx.resources.get_mut::<RenderAssets<TextBuffer>>().expect("TextBuffer render assets not found");
    let mut text_renderer = ctx.resources.get_mut::<TextRenderer>().expect("TextRenderer resource not found");
    let mut font_system = ctx.resources.get_mut::<FontSystem>().expect("FontSystem resource not found");
    let mut text_atlas = ctx.resources.get_mut::<TextAtlas>().expect("TextAtlas resource not found");
    let viewport = ctx.resources.get::<Viewport>().expect("Viewport resource not found");
    let mut swash_cache = ctx.resources.get_mut::<SwashCache>().expect("SwashCache resource not found");

    // add other node types as options 
    // TODO: implement Option<T> to query
    let mut text_query = query.cast::<&Text, With<Node>>();
    let ui_nodes = ui_nodes.into_iter().map(|(id, transform, global_transform, node, computed)| {
        let text = if let Some(text) = text_query.get(*id) {
            let text = text_buffers.get_by_entity(id, text, ctx);
            Some(text)
        } else {
            None
        };
        
        (id, transform, global_transform, node, computed, text)
    }).collect::<Vec<_>>();

    let mut text_areas = Vec::new();
    let mut ui_transforms = Vec::new();
    let mut transform_index = 0;
    let size = *ctx.renderer.size();

    for (id, transform, global_transform, node, computed, text) in &ui_nodes {
        // extract global translation
        let translation = global_transform.matrix.to_scale_rotation_translation().2;

        // add node to mesh
        if node.background_color != palette::TRANSPARENT && node.display != Display::None {
            ui_mesh.add_rect(
                // transform.translation.x,
                0.0,
                // size.height as f32 - transform.translation.y,
                0.0,
                computed.z_index as f32,
                computed.width.border,
                // -computed.height,
                computed.height.border,
                node.background_color,
                transform_index,
            );
        }
        
        // entitie's transform
        let mut glob_transform = global_transform.matrix.to_cols_array_2d();
        glob_transform[3][2] = computed.z_index as f32; // z-index as pos.z
        ui_transforms.push(glob_transform);
        transform_index += 1;

        // prepare text node for rendering
        if let Some(text) = text {
            // translation width the content box offset
            let content_translation = Vec2::new(
                translation.x + computed.width.offset(),
                translation.y + computed.height.offset(),
            );

            // TODO: computed.color
            text_areas.push(TextArea {
                buffer: &text.buffer,
                left: content_translation.x,
                top: content_translation.y,
                scale: 1.0,
                bounds: TextBounds {
                    left: content_translation.x as i32,
                    top: content_translation.y as i32,
                    right: (content_translation.x + computed.width.content) as i32,
                    bottom: (content_translation.y + computed.height.content) as i32,
                },
                default_color: palette::WHITE.into(),
                custom_glyphs: &[],
            })
        }
    }

    // prepare text areas for rendering
    text_renderer.prepare(
        ctx.renderer.device(), 
        ctx.renderer.queue(), 
        &mut font_system, 
        &mut text_atlas, 
        &viewport, 
        text_areas, 
        &mut swash_cache
    ).unwrap();

    // update transform storage with ui nodes
    ui_transform_storage.update(&ui_transforms, ui_transforms.len(), ctx);
}
