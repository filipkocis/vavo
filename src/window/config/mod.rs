mod cursor;
mod icon;

pub use cursor::*;
pub use icon::*;

use glam::IVec2;
use winit::window::{WindowAttributes, WindowButtons};

/// Configuration used when creating a window.
#[derive(crate::macros::Resource, Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    /// Size of the window.
    ///
    /// # Usage
    /// You can use `(u32, u32).into()` to convert a tuple into [`WindowResolution`].
    pub resolution: WindowResolution, 
    pub resize_constraints: WindowResizeConstraints,
    pub mode: WindowMode,

    pub cursor_mode: CursorMode,
    pub position: WindowPosition,
    pub enabled_buttons: EnabledButtons,
    pub window_level: WindowLevel,

    pub preferred_theme: PreferredTheme,
    pub icon: Icon,
    /// Cursor icon.
    ///
    /// # Usage
    /// For [`CursorIcon`] you can call `.into()` to convert it into a [`Cursor`],
    /// or call `from_rgba` / `from_path` to create a [`CustomCursor`] variant.
    pub cursor: Cursor,

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

impl WindowResolution {
    pub fn new(physical_width: u32, physical_height: u32, scale_factor: f64) -> Self {
        Self {
            physical_width,
            physical_height,
            scale_factor,
        }
    }
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

impl From<(u32, u32)> for WindowResolution {
    fn from(value: (u32, u32)) -> Self {
        Self {
            physical_width: value.0,
            physical_height: value.1,
            scale_factor: 1.0,
        } 
    }
}

impl From<WindowResolution> for Option<winit::dpi::Size> {
    fn from(value: WindowResolution) -> Self {
        Some(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(
            value.physical_width,
            value.physical_height,
        )))
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

impl From<EnabledButtons> for WindowButtons {
    fn from(value: EnabledButtons) -> Self {
        let mut buttons = WindowButtons::empty();
        buttons.set(WindowButtons::CLOSE, value.close);
        buttons.set(WindowButtons::MINIMIZE, value.minimize);
        buttons.set(WindowButtons::MAXIMIZE, value.maximize);
        buttons 
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

impl From<WindowLevel> for winit::window::WindowLevel {
    fn from(value: WindowLevel) -> Self {
        match value {
            WindowLevel::AlwaysOnBottom => Self::AlwaysOnBottom,
            WindowLevel::Normal => Self::Normal,
            WindowLevel::AlwaysOnTop => Self::AlwaysOnTop,
        } 
    }
}

/// See [`winit::window::Theme`].
#[derive(Default, Debug, Clone, Copy)]
pub enum PreferredTheme {
    #[default]
    None,
    Light,
    Dark,
}

impl From<PreferredTheme> for Option<winit::window::Theme> {
    fn from(value: PreferredTheme) -> Self {
        match value {
            PreferredTheme::None => None,
            PreferredTheme::Light => Some(winit::window::Theme::Light),
            PreferredTheme::Dark => Some(winit::window::Theme::Dark),
        } 
    }
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
            icon: Icon::None,
            cursor: Default::default(),

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

impl WindowConfig {
    pub fn get_window_attributes(&self) -> WindowAttributes {
        let mut attrs = WindowAttributes::default();

        attrs.inner_size = self.resolution.into();
        // min_inner_size: None,
        // max_inner_size: None,
        // position: None,
        attrs.resizable = self.resizable;
        attrs.enabled_buttons = self.enabled_buttons.into();
        attrs.title = self.title.clone();
        attrs.maximized = self.maximized;
        // fullscreen: None,
        attrs.visible = self.visible;
        attrs.transparent = self.transparent;
        attrs.blur = self.blur;
        attrs.decorations = self.decorations;
        attrs.window_level = self.window_level.into();
        // window_icon: None,
        attrs.preferred_theme = self.preferred_theme.into();
        attrs.content_protected = self.content_protected;
        // cursor: Cursor::default(),
        attrs.active = self.active;

        attrs
    }
}
