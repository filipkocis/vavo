use glam::{Mat4, Vec4Swizzles};

use crate::{
    core::lighting::LightAndShadowManager,
    math::CubeFace,
    prelude::*,
    renderer::newtype::{RenderDevice, RenderQueue},
};

/// Prepared light data for rendering
#[derive(crate::macros::Resource)]
pub struct PreparedLightData {
    pub lights: Vec<Light>,
}

/// Pre-render system to prepare [`light data`](PreparedLightData) resource for rendering
pub fn prepare_light_data_system(
    world: &mut World,
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    ambient_light: Option<Res<AmbientLight>>,
    mut light_manager: ResMut<LightAndShadowManager>,

    mut camera_query: Query<(&GlobalTransform, &Camera), (With<Projection>, With<Camera3D>)>,
    mut directional_query: Query<(&GlobalTransform, &DirectionalLight)>,
    mut spot_query: Query<(&GlobalTransform, &SpotLight)>,
    mut point_query: Query<(&GlobalTransform, &PointLight)>,
) {
    // Extract camera position
    let active_camera = camera_query
        .iter_mut()
        .into_iter()
        .filter(|(_, c)| c.active)
        .take(1)
        .next();
    let camera_position = match active_camera.map(|(t, _)| t.matrix.w_axis.xyz()) {
        Some(p) => p,
        None => {
            commands.insert_resource(PreparedLightData { lights: Vec::new() });
            return;
        }
    };

    let mut lights = Vec::new();

    // directional lights
    for (global_transform, light) in directional_query.iter_mut() {
        let (view_projection_matrix, direction) =
            light.view_projection_matrix(50.0, 0.1, 50.0, camera_position, global_transform.matrix);

        lights.push(
            light
                .as_light(view_projection_matrix)
                .with_directional(direction),
        )
    }

    // spot lights
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
    if let Some(light) = ambient_light {
        lights.push(light.as_light(Mat4::IDENTITY))
    };

    light_manager.update(&mut lights, world, &device, &queue);

    let prepared_light_data = PreparedLightData { lights };
    commands.insert_resource(prepared_light_data);
}
