use glam::{EulerRot, Quat, Vec3};
use winit::keyboard::KeyCode;

use crate::prelude::*;

pub fn movement_system(ctx: &mut SystemsContext, mut query: Query<(&mut Transform, &mut Projection, &Camera), With<Camera3D>>) {
    let time = ctx.resources.get::<Time>().unwrap(); 
    let key_input = ctx.resources.get::<Input<KeyCode>>().unwrap(); 
    let mouse_motion = ctx.event_reader.read::<MouseMotion>();

    // Camera translation
    let mut pos_dx = 0.0;
    let mut pos_dy = 0.0;
    let mut pos_dz = 0.0;
    
    if key_input.pressed(KeyCode::KeyW) {
        pos_dz -= 0.1;
    }
    if key_input.pressed(KeyCode::KeyS) {
        pos_dz += 0.1;
    }
    if key_input.pressed(KeyCode::KeyA) {
        pos_dx -= 0.1;
    }
    if key_input.pressed(KeyCode::KeyD) {
        pos_dx += 0.1;
    }
    if key_input.pressed(KeyCode::Space) && key_input.pressed(KeyCode::ShiftLeft) {
        pos_dy -= 0.1;
    } else if key_input.pressed(KeyCode::Space) {
        pos_dy += 0.1;
    }

    // Camera rotation
    let mut rot_dy = 0.0;
    let mut rot_dx = 0.0;

    for motion in mouse_motion {
        rot_dx -= motion.delta.x;
        rot_dy -= motion.delta.y;
    }

    let sensitivity = 0.1;
    rot_dy *= sensitivity;
    rot_dx *= sensitivity;

    if rot_dx == 0.0 && rot_dy == 0.0 && pos_dx == 0.0 && pos_dz == 0.0 && pos_dy == 0.0 {
        return;
    }

    // return;
    for (transform, _projection, camera) in query.iter_mut() {
        if !camera.active {
            return
        }
        
        let rotation = transform.rotation;
        let forward = rotation * Vec3::Z;
        let right = rotation * Vec3::X;

        let speed = 100.0;
        transform.translation += (forward * pos_dz + right * pos_dx) * time.delta() * speed;
        transform.translation.y += pos_dy * time.delta() * speed;

        // Compute local pitch (vertical rotation)
        let pitch = transform.rotation.to_euler(EulerRot::YXZ).1; // Extract current X-axis angle
        let max_pitch = 89.0_f32.to_radians();
        let new_pitch = (pitch + rot_dy.to_radians()).clamp(-max_pitch, max_pitch);

        let global_y_rotation = Quat::from_rotation_y(rot_dx.to_radians());
        let local_x_rotation = Quat::from_rotation_x(new_pitch - pitch);

        // Apply the rotations
        transform.rotation = global_y_rotation * transform.rotation; // Rotate around global Y
        transform.rotation = transform.rotation * local_x_rotation;  

        // match projection {
        //     Projection::Perspective(proj) => {
        //         if key_input.pressed(KeyCode::KeyQ) {
        //             proj.fov -= 0.1;
        //         } 
        //         if key_input.pressed(KeyCode::KeyE) {
        //             proj.fov += 0.1;
        //         }
        //     }
        //     _ => {}
        // }

        return;
    }
}
