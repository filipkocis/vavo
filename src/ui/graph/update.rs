use glam::Vec2;
use glyphon::{
    FontSystem, Resolution, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use winit::event::WindowEvent;

use crate::prelude::*;
use crate::render_assets::RenderAssets;
use crate::ui::{graph::storage::UiTransformStorage, mesh::*, prelude::*, text::TextBuffer};

/// System to update the glyphon text viewport resolution.
/// Runs only if the window size has changed.
pub fn update_glyphon_viewport(ctx: &mut SystemsContext, _: Query<()>) {
    let mut viewport = ctx.resources.get_mut::<Viewport>();
    let queue = ctx.renderer.queue();
    let size = ctx.renderer.size();

    viewport.update(
        queue,
        Resolution {
            width: size.width,
            height: size.height,
        },
    )
}

/// Utility function to check for a window resize event.
pub fn has_resized(ctx: &SystemsContext) -> bool {
    ctx.event_reader
        .read::<WindowEvent>()
        .iter()
        .any(|event| matches!(event, WindowEvent::Resized(_)))
}

/// Clear glyphon's text_renderer. Used when all nodes are removed.
fn clear_text_renderer(ctx: &mut SystemsContext) {
    let mut text_renderer = ctx.resources.get_mut::<TextRenderer>();
    let mut font_system = ctx.resources.get_mut::<FontSystem>();
    let mut text_atlas = ctx.resources.get_mut::<TextAtlas>();
    let viewport = ctx.resources.get::<Viewport>();
    let mut swash_cache = ctx.resources.get_mut::<SwashCache>();

    text_renderer
        .prepare(
            ctx.renderer.device(),
            ctx.renderer.queue(),
            &mut font_system,
            &mut text_atlas,
            &viewport,
            [],
            &mut swash_cache,
        )
        .unwrap();
}

// TODO: add tracking system when only some nodes get removed, because now it will not trigger the update.
// Implement Local<T> storage or Removed<T> filter/resource for this to work.

/// System to update the UI mesh and UI transform storage, runs only if some nodes have `Changed<Transform>` filter.
/// The filter should return true after `compute_nodes_and_transforms` system which runs on `Changed<Node>` filter and updates transforms.
///
/// # Resize
/// It will run on window resize even if no nodes have changed. That is because glyphon text gets
/// automatically clipped so we need to update prepared text areas.
///
/// # Note
/// Applies z-index to the z component of the global transfrom pushed to the transform storage.
pub fn update_ui_mesh_and_transforms(ctx: &mut SystemsContext, mut query: Query<()>) {
    // resources
    let mut ui_transform_storage = ctx.resources.get_mut::<UiTransformStorage>();
    let mut ui_mesh = ctx.resources.get_mut::<UiMesh>();
    let mut ui_mesh_transparent = ctx.resources.get_mut::<UiMeshTransparent>();
    let mut ui_mesh_images = ctx.resources.get_mut::<UiMeshImages>();

    // get the amount of changed nodes
    let mut changed_query = query.cast::<EntityId, (
        With<Transform>,
        With<GlobalTransform>,
        With<Node>,
        With<ComputedNode>,
        Changed<Transform>,
    )>();
    let changed_len = changed_query.iter_mut().len();

    // query all nodes
    let mut nodes_query = query.cast::<(
        EntityId,
        &GlobalTransform,
        &Node,
        &ComputedNode,
        Option<&Text>,
        Option<&UiImage>,
    ), ()>();
    let ui_nodes = nodes_query.iter_mut();

    // return if nothing changed
    let resized = has_resized(ctx);
    if changed_len == 0 && !resized {
        // cleanup if all nodes were removed
        if ui_nodes.is_empty() && !ui_mesh.positions.is_empty() {
            ui_mesh.clear();
            ui_mesh_transparent.clear();
            ui_mesh_images.clear();
            clear_text_renderer(ctx);
        }
        return;
    }

    // clear the mesh and regenerate all nodes
    // TODO: optimize this
    ui_mesh.clear();
    ui_mesh_transparent.clear();
    ui_mesh_images.clear();

    // text resources
    let mut text_buffers = ctx.resources.get_mut::<RenderAssets<TextBuffer>>();
    let mut text_renderer = ctx.resources.get_mut::<TextRenderer>();
    let mut font_system = ctx.resources.get_mut::<FontSystem>();
    let mut text_atlas = ctx.resources.get_mut::<TextAtlas>();
    let viewport = ctx.resources.get::<Viewport>();
    let mut swash_cache = ctx.resources.get_mut::<SwashCache>();

    // intermediate storage for text buffer raes
    let mut intermediate_text_rae = Vec::new();

    // add other node types as options
    let ui_nodes = ui_nodes
        .into_iter()
        .map(|(id, global_transform, node, computed, text, image)| {
            // HINT: if node has text, get the text buffer rae, add it to intermediate storage for RefCell
            // lifetime issues, then later in code retrieve it and push its borrow to text_borrows
            if let Some(text) = text {
                let text = text_buffers.get_by_entity(id, text, ctx);
                intermediate_text_rae.push(Some(text));
            } else {
                intermediate_text_rae.push(None);
            };

            let has_image = image.is_some();

            // return core ui node
            (id, global_transform, node, computed, has_image)
        })
        .collect::<Vec<_>>();

    // borrow intermediate text raes, needed for lifetime issues
    let text_borrows = intermediate_text_rae
        .iter()
        .map(|rae_option| rae_option.as_ref().map(|rae| rae.buffer.lock().unwrap()))
        .collect::<Vec<_>>();

    let mut text_areas = Vec::new();
    let mut ui_transforms = Vec::new();
    let mut transform_index = 0;

    for (i, (id, global_transform, node, computed, has_image)) in ui_nodes.into_iter().enumerate() {
        // extract global translation
        let translation = global_transform.translation();

        // dont add node to mesh
        if node.display == Display::None {
            continue;
        }

        let horizontal = computed.border.horizontal();
        let vertical = computed.border.vertical();

        // x, y, w, h, color
        let quads = [
            // content + padding
            (
                computed.border.left,
                computed.border.top,
                computed.width.border - horizontal,
                computed.height.border - vertical,
                node.background_color,
                has_image,
            ),
            // top border
            (
                0.0,
                0.0,
                computed.width.border,
                computed.border.top,
                node.border_color,
                false,
            ),
            // left border
            (
                0.0,
                0.0,
                computed.border.left,
                computed.height.border,
                node.border_color,
                false,
            ),
            // right border
            (
                computed.width.border - computed.border.right,
                0.0,
                computed.border.right,
                computed.height.border,
                node.border_color,
                false,
            ),
            // bottom border
            (
                0.0,
                computed.height.border - computed.border.bottom,
                computed.width.border,
                computed.border.bottom,
                node.border_color,
                false,
            ),
        ];

        // add quad with borders to mesh
        for (x, y, w, h, color, has_image) in quads {
            if w > 0.0 && h > 0.0 && color.a > 0.0 {
                if color.a == 1.0 {
                    ui_mesh.add_rect(
                        x,
                        y,
                        computed.z_index as f32,
                        w,
                        h,
                        color,
                        transform_index,
                        id,
                    );
                } else {
                    ui_mesh_transparent.add_rect(
                        x,
                        y,
                        computed.z_index as f32,
                        w,
                        h,
                        color,
                        transform_index,
                        id,
                    );
                }
            }

            if w > 0.0 && h > 0.0 && has_image {
                ui_mesh_images.add_rect(
                    x,
                    y,
                    computed.z_index as f32,
                    w,
                    h,
                    color::WHITE,
                    transform_index,
                    id,
                );
            }
        }

        // entitie's transform
        let glob_transform = global_transform.matrix.to_cols_array_2d();
        // TODO: we should use this instead of vec3.z in pos, but for nonui parent nodes this would
        // not work, so implement it later
        // glob_transform[3][2] = computed.z_index as f32; // z-index as pos.z
        ui_transforms.push(glob_transform);
        transform_index += 1;

        // prepare text node for rendering
        if let Some(text) = &text_borrows[i] {
            // translation with the content box offset
            let content_translation = Vec2::new(
                translation.x + computed.width.offset(),
                translation.y + computed.height.offset(),
            );

            text_areas.push(TextArea {
                buffer: text,
                left: content_translation.x,
                top: content_translation.y,
                scale: 1.0,
                bounds: TextBounds {
                    left: content_translation.x as i32,
                    top: content_translation.y as i32,
                    right: (content_translation.x + computed.width.content) as i32,
                    bottom: (content_translation.y + computed.height.content) as i32,
                },
                default_color: computed.color.into(),
                custom_glyphs: &[],
            })
        }
    }

    // prepare text areas for rendering
    text_renderer
        .prepare_with_depth(
            ctx.renderer.device(),
            ctx.renderer.queue(),
            &mut font_system,
            &mut text_atlas,
            &viewport,
            text_areas,
            &mut swash_cache,
            |md| {
                // TODO: do a better way to match with UI shader, this is copypasting
                let mil = 1_000_000.0;
                (mil - md as f32 - 1.0) / mil
            },
        )
        .unwrap();

    // update transform storage with ui nodes
    ui_transform_storage.update(&ui_transforms, ui_transforms.len(), ctx);
}
