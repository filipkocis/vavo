mod app_handler;
pub mod config;
mod render;
mod state;

pub(crate) use app_handler::AppHandler;
pub(crate) use render::*;
pub(crate) use state::*;

/// Resource holding basic window state information.
/// TODO: This struct is temporary and will be expanded in the future. It may eventually replace
/// [WindowConfig](config::WindowConfig).
#[derive(crate::macros::Resource, Default, Debug, Clone)]
pub struct Window {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub cursor_position: Option<glam::Vec2>,
}

pub mod prelude {
    pub use super::Window;
}
