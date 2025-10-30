use glam::{Mat4, Vec4Swizzles};

use crate::{core::lighting::LightAndShadowManager, math::CubeFace, prelude::*};

/// Prepared light data for rendering
#[derive(crate::macros::Resource)]
pub struct PreparedLightData {
    pub lights: Vec<Light>,
}

impl PreparedLightData {
    /// Returns prepared light data, requires lights and shadow manager to be a valid resource.
    /// Should be called before rendering and set as a resource.
    pub fn prepare(ctx: &mut SystemsContext, mut query: Query<()>) -> Self {
        // Extract camera position
        let mut camera_query =
            query.cast::<(&GlobalTransform, &Camera), (With<Projection>, With<Camera3D>)>();
        let active_camera = camera_query
            .iter_mut()
            .into_iter()
            .filter(|(_, c)| c.active)
            .take(1)
            .next();
        let camera_position = match active_camera.map(|(t, _)| t.matrix.w_axis.xyz()) {
            Some(p) => p,
            None => return Self { lights: Vec::new() },
        };

        let mut lights = Vec::new();

        // directional lights
        let mut directional_query = query.cast::<(&GlobalTransform, &DirectionalLight), ()>();
        for (global_transform, light) in directional_query.iter_mut() {
            let (view_projection_matrix, direction) = light.view_projection_matrix(
                50.0,
                0.1,
                50.0,
                camera_position,
                global_transform.matrix,
            );

            lights.push(
                light
                    .as_light(view_projection_matrix)
                    .with_directional(direction),
            )
        }

        // spot lights
        let mut spot_query = query.cast::<(&GlobalTransform, &SpotLight), ()>();
        for (global_transform, light) in spot_query.iter_mut() {
            let (view_projection_matrix, spot_direction) =
                light.view_projection_matrix(1.0, 0.1, global_transform.matrix);

            lights.push(
                light
                    .as_light(view_projection_matrix)
                    .with_spot(global_transform.matrix.w_axis.xyz(), spot_direction),
            )
        }

        // point lights
        let mut point_query = query.cast::<(&GlobalTransform, &PointLight), ()>();
        for (global_transform, light) in point_query.iter_mut() {
            for i in 0..6 {
                let face = CubeFace::from_index(i);
                let view_projection_matrix =
                    light.view_proj_matrix_for_face(global_transform.matrix, face);

                lights.push(
                    light
                        .as_light(view_projection_matrix)
                        .with_point(global_transform.matrix.w_axis.xyz()),
                )
            }
        }

        // ambient light
        if let Some(light) = ctx.resources.try_get::<AmbientLight>() {
            lights.push(light.as_light(Mat4::IDENTITY))
        };

        let mut light_manager = ctx.resources.get_mut::<LightAndShadowManager>();
        light_manager.update(&mut lights, ctx);

        Self { lights }
    }
}
