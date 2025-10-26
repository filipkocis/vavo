mod cursor;
mod icon;

pub use cursor::*;
pub use icon::*;

use glam::IVec2;
use winit::{
    dpi::{LogicalSize, Size},
    window::{Fullscreen, WindowAttributes, WindowButtons},
};

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

impl WindowResizeConstraints {
    pub fn into_min_size(&self) -> Option<Size> {
        if self.min_width == 0.0 && self.min_height == 0.0 {
            return None;
        }

        Some(Size::Logical(LogicalSize::new(
            self.min_width as f64,
            self.min_height as f64,
        )))
    }

    pub fn into_max_size(&self) -> Option<Size> {
        if self.max_width == f32::INFINITY && self.max_height == f32::INFINITY {
            return None;
        }

        Some(Size::Logical(LogicalSize::new(
            self.max_width as f64,
            self.max_height as f64,
        )))
    }
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

impl WindowMode {
    pub fn into_winit_fullscreen(&self, window: &winit::window::Window) -> Option<Fullscreen> {
        match self {
            Self::Windowed => None,
            Self::Borderless => Some(Fullscreen::Borderless(None)),
            Self::Fullscreen => Some(Fullscreen::Exclusive({
                let monitor = match (
                    window.current_monitor(),
                    window.primary_monitor(),
                    window.available_monitors().next(),
                ) {
                    (Some(monitor), _, _) => monitor,
                    (_, Some(monitor), _) => monitor,
                    (_, _, Some(monitor)) => monitor,
                    _ => {
                        eprintln!("No monitor found, falling back to windowed mode");
                        return None;
                    }
                };

                let Some(video_mode_handle) = monitor
                    .video_modes()
                    // TODO: Is this necessary?
                    .max_by_key(|mode| {
                        let w = mode.size().width as u64;
                        let h = mode.size().height as u64;
                        let r = mode.refresh_rate_millihertz() as u64;

                        w.saturating_mul(h).saturating_mul(r)
                    })
                else {
                    eprintln!("No video mode found, falling back to windowed mode");
                    return None;
                };

                video_mode_handle
            })),
        }
    }
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

impl From<CursorGrabMode> for winit::window::CursorGrabMode {
    fn from(value: CursorGrabMode) -> Self {
        match value {
            CursorGrabMode::None => Self::None,
            CursorGrabMode::Confined => Self::Confined,
            CursorGrabMode::Locked => Self::Locked,
        }
    }
}

/// See `position` as defined in [`winit::window::WindowAttributes`]
#[derive(Default, Debug, Clone, Copy)]
pub enum WindowPosition {
    #[default]
    Auto,
    Centered,
    Physical(IVec2),
}

impl From<WindowPosition> for Option<winit::dpi::Position> {
    fn from(value: WindowPosition) -> Self {
        match value {
            WindowPosition::Auto => None,
            WindowPosition::Centered => unimplemented!("WindowPosition::Centered"),
            WindowPosition::Physical(pos) => Some(winit::dpi::Position::Physical(
                winit::dpi::PhysicalPosition::new(pos.x, pos.y),
            )),
        }
    }
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
        attrs.min_inner_size = self.resize_constraints.into_min_size();
        attrs.max_inner_size = self.resize_constraints.into_max_size();
        attrs.position = self.position.into();
        attrs.resizable = self.resizable;
        attrs.enabled_buttons = self.enabled_buttons.into();
        attrs.title = self.title.clone();
        attrs.maximized = self.maximized;
        // attrs.fullscreen = self.mode.into();
        attrs.visible = self.visible;
        attrs.transparent = self.transparent;
        attrs.blur = self.blur;
        attrs.decorations = self.decorations;
        attrs.window_level = self.window_level.into();
        attrs.window_icon = self.icon.clone().into();
        attrs.preferred_theme = self.preferred_theme.into();
        attrs.content_protected = self.content_protected;
        // attrs.cursor = self.cursor.clone().into();
        attrs.active = self.active;

        attrs
    }

    pub fn post_apply(
        &self,
        window: &winit::window::Window,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        // fullscreen
        let fullscreen = self.mode.into_winit_fullscreen(window);
        window.set_fullscreen(fullscreen);

        // cursor
        let cursor = self.cursor.into_winit_cursor(event_loop);
        window.set_cursor(cursor);

        // cursor mode
        let grab_mode = self.cursor_mode.grab_mode.into();
        if let Err(err) = window.set_cursor_grab(grab_mode) {
            eprintln!("Failed to set cursor grab mode: {}", err);
        };
        window.set_cursor_visible(self.cursor_mode.visible);
    }
}
