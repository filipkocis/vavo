use crate::{app::{App, Plugin}, core::standard::{movement::movement_system, prepare::graph_prerender_preparation_system, startup::{add_render_resources, register_standard_graph}, update::{update_camera_buffers, update_global_transforms}}, input::InputPlugin, prelude::{FixedTime, Time}, system::{System, SystemStage}, ui::graph::UiPlugin};

/// Default plugins which are necessary for the app to run, includes:
/// - `RenderPlugin` 
/// - `TimePlugin`
/// - `InputPlugin`
/// - `UiPlugin`
pub struct DefaultPlugin;

impl Plugin for DefaultPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(RenderPlugin)
            .add_plugin(TimePlugin)
            .add_plugin(InputPlugin)
            .add_plugin(UiPlugin);
    }
}

/// Provides rendering functionality to the app, with standard render graph and other necessary
/// systems.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(add_render_resources)
            .add_startup_system(register_standard_graph)
            .register_system(update_global_transforms, SystemStage::Last)
            .register_system(update_camera_buffers, SystemStage::PreRender)
            .register_system(graph_prerender_preparation_system, SystemStage::PreRender);
    }
}

/// Provides default camera movement functionality, good when no proper movement system is implemented yet.
pub struct NoclipMovementPlugin;

impl Plugin for NoclipMovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(movement_system);
    }
}

/// Adds time functionality to the app via the `Time` resource.
pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.world.resources.insert(Time::new());
        app.world.resources.insert(FixedTime::from_hz(60.0));

        let time = app.world.resources.get::<Time>().unwrap();
        app.world.entities.initialize_tick(time.tick_raw());
    }
}
