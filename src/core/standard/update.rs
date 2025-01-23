use winit::event::WindowEvent;

use crate::{prelude::*, render_assets::*};

/// Internal system that updates active camera buffers with changed projection and transform.
pub fn update_camera_buffers<'a>(
    ctx: &mut SystemsContext, 
    mut query: Query< 
        (&'a EntityId, &'a Camera, &'a Projection, &'a GlobalTransform), 
        // TODO: add OR<T> since camera needs both proj and trans to be mutated to update
        // (With<Camera3D>, Changed<Projection>, Changed<Transform>)
        (With<Camera3D>, With<Projection>, Changed<GlobalTransform>)
    >
) {
    let mut buffers = ctx.resources.get_mut::<RenderAssets<Buffer>>().unwrap();
    let resize_event = ctx.event_reader.read::<WindowEvent>()
        .into_iter().filter_map(|e| {
            if let WindowEvent::Resized(size) = e {
                Some(*size)
            } else {
                None
            }
        }).last();

    if let Some(size) = resize_event {
        let mut proj_query = query.cast::<&mut Projection, With<Camera>>();
        for proj in proj_query.iter_mut() {
            proj.resize(size.width as f32, size.height as f32);
        }
    }

    for (id, camera, projection, global_transform) in query.iter_mut() {
        if !camera.active {
            continue
        }

        let camera_buffer = buffers.get_by_entity(id, camera, ctx);
        let camera_buffer_data = Camera::get_buffer_data(projection, global_transform);

        let camera_buffer = camera_buffer.uniform.as_ref().expect("Camera buffer should be an uniform buffer");
        let data = bytemuck::cast_slice(&camera_buffer_data);
        
        ctx.renderer.queue().write_buffer(camera_buffer, 0, data);
    }
}

/// Internal system that updates global transforms of entities with changed local transforms.
pub fn update_global_transforms(_: &mut SystemsContext, mut q: Query<()>) {
    // update root entities
    let mut query = q.cast::<(&mut GlobalTransform, &Transform), (Changed<Transform>, Without<Parent>)>();
    for (global, local) in query.iter_mut() {
        global.update(local);
    }

    // recursively update children of updated entities
    let mut query = q.cast::<(&EntityId, &mut GlobalTransform), (With<Children>, Changed<Transform>)>();
    for (id, global) in query.iter_mut() {
        update_children(*id, global, q.cast());
    }
}

fn update_children(parent_id: EntityId, parent_global: &GlobalTransform, mut parent_query: Query<&Children>) {
    // get children of parent
    let children = match parent_query.get(parent_id) {
        Some(children) => children,
        None => return,
    };

    // update every child recursively
    let mut child_query = parent_query.cast::<(&mut GlobalTransform, &Transform), With<Parent>>();
    for child in &children.ids {
        if let Some((global, local)) = child_query.get(*child) {
            // update child of parent
            *global = parent_global.combine_child(local);

            // recursively update children of child
            update_children(*child, global, child_query.cast());
        } 
    }
}
