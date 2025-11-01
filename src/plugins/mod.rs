use crate::{
    app::{App, Plugin},
    audio::AudioPlugin,
    core::standard::{
        grouped::generate_grouped_instances_system,
        light_data::prepare_light_data_system,
        movement::movement_system,
        startup::{add_render_resources, register_standard_graph},
        update::{update_camera_buffers, update_global_transforms},
    },
    event::plugin::EventPlugin,
    input::InputPlugin,
    prelude::{FixedTime, Time},
    reflect::ReflectionPlugin,
    renderer::culling::FrustumCullingPlugin,
    system::SystemStage,
    ui::plugin::UiPlugin,
};

/// Default plugins which are necessary for the app to run, includes:
/// - [`EventPlugin`]
/// - [`RenderPlugin`]
/// - [`TimePlugin`]
/// - [`InputPlugin`]
/// - [`UiPlugin`]
/// - [`AudioPlugin`]
/// - [`ReflectionPlugin`]
/// - [`FrustumCullingPlugin`]
pub struct DefaultPlugin;

impl Plugin for DefaultPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EventPlugin)
            .add_plugin(RenderPlugin)
            .add_plugin(TimePlugin)
            .add_plugin(InputPlugin)
            .add_plugin(UiPlugin)
            .add_plugin(AudioPlugin)
            .add_plugin(ReflectionPlugin)
            .add_plugin(FrustumCullingPlugin);
    }
}

// TODO: move these plugins to their own files

/// Provides rendering functionality to the app, with standard render graph and other necessary
/// systems.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_render_resources)
            .add_startup_system(register_standard_graph)
            .register_system(update_global_transforms, SystemStage::Last)
            .register_system(update_camera_buffers, SystemStage::PreRender)
            .register_system(prepare_light_data_system, SystemStage::PreRender)
            .register_system(generate_grouped_instances_system, SystemStage::PreRender);
    }
}

/// Provides default camera movement functionality, good when no proper movement system is implemented yet.
pub struct NoclipMovementPlugin;

impl Plugin for NoclipMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement_system);
    }
}

/// Adds time functionality to the app via the `Time` resource.
pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.world.resources.insert(Time::new());
        app.world.resources.insert(FixedTime::from_hz(60.0));
    }
}
