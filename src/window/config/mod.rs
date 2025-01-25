mod cursor;
mod icon;

pub use cursor::*;
pub use icon::*;

use glam::IVec2;

/// Configuration used when creating a window.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub resolution: WindowResolution, 
    pub resize_constraints: WindowResizeConstraints,
    pub mode: WindowMode,

    pub cursor_mode: CursorMode,
    pub position: WindowPosition,
    pub enabled_buttons: EnabledButtons,
    pub window_level: WindowLevel,

    pub preferred_theme: PreferredTheme,
    pub icon: WindowIcon,

    pub resizable: bool,
    pub maximized: bool,
    pub visible: bool,
    pub transparent: bool,
    pub blur: bool,
    pub decorations: bool,
    pub content_protected: bool,
    pub active: bool,
}

/// See `inner_size` as defined in [`winit::window::WindowAttributes`]
#[derive(Debug, Clone, Copy)]
pub struct WindowResolution {
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
}

impl Default for WindowResolution {
    fn default() -> Self {
        Self {
            physical_width: 1280,
            physical_height: 720,
            scale_factor: 1.0,
        }
    }
}

/// See `min_inner_size` and `max_inner_size` as defined in [`winit::window::WindowAttributes`]
#[derive(Debug, Clone, Copy)]
pub struct WindowResizeConstraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Default for WindowResizeConstraints {
    fn default() -> Self {
        Self {
            min_width: 180.0,
            min_height: 120.0,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        }
    }
}

/// See [`winit::window::Fullscreen`].
#[derive(Default, Debug, Clone, Copy)]
pub enum WindowMode {
    #[default]
    Windowed,
    Fullscreen,
    Borderless,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorMode {
    pub grab_mode: CursorGrabMode,
    pub visible: bool,
}

impl Default for CursorMode {
    fn default() -> Self {
        Self {
            grab_mode: Default::default(),
            visible: true,
        }
    }
}

/// See [`winit::window::CursorGrabMode`].
#[derive(Default, Debug, Clone, Copy)]
pub enum CursorGrabMode {
    #[default]
    None,
    Confined,
    Locked,
}

/// See `position` as defined in [`winit::window::WindowAttributes`]
#[derive(Default, Debug, Clone, Copy)]
pub enum WindowPosition {
    #[default]
    Auto,
    Centered,
    Physical(IVec2),
}

/// See [`winit::window::WindowButtons`].
#[derive(Debug, Clone, Copy)]
pub struct EnabledButtons {
    pub close: bool,
    pub minimize: bool,
    pub maximize: bool,
}

impl EnabledButtons {
    /// Enable all buttons.
    pub fn all() -> Self {
        Self {
            close: true,
            minimize: true,
            maximize: true,
        }
    }

    /// Disable all buttons.
    pub fn none() -> Self {
        Self {
            close: false,
            minimize: false,
            maximize: false,
        }
    }
}

impl Default for EnabledButtons {
    fn default() -> Self {
        Self::all()
    }
}

/// See [`winit::window::WindowLevel`].
#[derive(Default, Debug, Clone, Copy)]
pub enum WindowLevel {
    AlwaysOnBottom,
    #[default]
    Normal,
    AlwaysOnTop,
}

/// See [`winit::window::Theme`].
#[derive(Default, Debug, Clone, Copy)]
pub enum PreferredTheme {
    #[default]
    None,
    Light,
    Dark,
}

impl Default for WindowConfig {
    #[inline]
    fn default() -> Self {
        Self {
            title: "vavo window".to_owned(),
            resolution: Default::default(),
            resize_constraints: Default::default(),
            mode: Default::default(),

            cursor_mode: Default::default(),
            position: Default::default(),
            enabled_buttons: EnabledButtons::all(),
            window_level: Default::default(),

            preferred_theme: PreferredTheme::None,
            icon: WindowIcon::None,

            resizable: true,
            maximized: false,
            visible: true,
            transparent: false,
            blur: false,
            decorations: true,
            content_protected: false,
            active: true,
        }
    }
}
