use crate::{app::{App, Plugin}, core::standard::{movement::movement_system, prepare::graph_prerender_preparation_system, startup::{add_render_resources, register_standard_graph}, update::{update_camera_buffers, update_global_transforms}}, system::{System, SystemStage}};

pub struct DefaultPlugin;

impl Plugin for DefaultPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(RenderPlugin);
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(System::new("add_render_resources", add_render_resources))
            .add_startup_system(System::new("register_standard_graph", register_standard_graph))
            .register_system(System::new("update_global_transforms", update_global_transforms), SystemStage::PostUpdate)
            .register_system(System::new("update_camera_buffers", update_camera_buffers), SystemStage::PreRender)
            .register_system(System::new("prepare_render_resources", graph_prerender_preparation_system), SystemStage::PreRender);
    }
}

pub struct NoclipMovementPlugin;

impl Plugin for NoclipMovementPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(System::new("movement_system", movement_system));
    }
}
