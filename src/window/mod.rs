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
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    pub(crate) cursor_position: Option<glam::Vec2>,
}

impl Window {
    /// Returns the current size of the window in physical pixels.
    #[inline]
    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    /// Returns the current cursor position within the window.
    #[inline]
    pub fn cursor_position(&self) -> Option<glam::Vec2> {
        self.cursor_position
    }
}

pub mod prelude {
    pub use super::Window;
}
