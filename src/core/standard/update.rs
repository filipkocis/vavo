use winit::event::WindowEvent;

use crate::{prelude::*, render_assets::*};

pub fn update_camera_buffers<'a>(
    ctx: &mut SystemsContext, 
    mut query: Query<'a, 
        (&'a EntityId, &'a Camera, &'a Projection, &'a Transform), 
        // TODO: add OR<T> since camera needs both proj and trans to be mutated to update
        // (With<Camera3D>, Changed<Projection>, Changed<Transform>)
        (With<Camera3D>, With<Projection>, Changed<Transform>)
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

    for (id, camera, projection, transform) in query.iter_mut() {
        if !camera.active {
            continue
        }

        let camera_buffer = buffers.get_by_entity(id, camera, ctx);
        let camera_buffer_data = Camera::get_buffer_data(projection, transform);

        let camera_buffer = camera_buffer.uniform.as_ref().expect("Camera buffer should be uniform");
        let data = bytemuck::cast_slice(&camera_buffer_data);
        
        ctx.renderer.queue().write_buffer(camera_buffer, 0, data);
    }
}
