use std::time::Duration;

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
    prelude::{FixedTime, FpsCounter, ResMut, Time, on_internval},
    reflect::ReflectionPlugin,
    renderer::culling::FrustumCullingPlugin,
    system::{IntoSystem, phase},
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
            .register_system(update_global_transforms, phase::Last)
            .register_system(update_camera_buffers, phase::PreRender)
            .register_system(prepare_light_data_system, phase::PreRender)
            .register_system(generate_grouped_instances_system, phase::PreRender);
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

/// Adds an FPS counter resource to the app
pub struct FpsCounterPlugin {
    /// The capacity of the FPS counter (number of samples to keep)
    pub capacity: usize,
    /// The interval (in seconds) at which to print the FPS to the console, or None to disable
    /// printing
    pub interval: Option<f32>,
}

impl Plugin for FpsCounterPlugin {
    fn build(&self, app: &mut App) {
        app.world.resources.insert(FpsCounter::new(self.capacity));
        app.add_system(update_fps_counter_system);

        if let Some(interval) = self.interval {
            let duration = Duration::from_secs_f32(interval);
            app.add_system(print_fps_system.run_if(on_internval(duration)));
        }
    }
}

/// System to update the FPS counter each frame
fn update_fps_counter_system(mut fps_counter: ResMut<FpsCounter>) {
    fps_counter.update();
}

/// System to print the current FPS to the console
fn print_fps_system(fps_counter: ResMut<FpsCounter>) {
    println!("FPS: {:.2}", fps_counter.average_fps());
}
