use std::path::Path;

pub use winit::window::CursorIcon;

#[derive(Clone, Debug)]
/// See [`winit::window::Cursor`].
pub enum Cursor {
    Icon(CursorIcon),
    Custom(CustomCursor),
}

impl Cursor {
    /// Load custom cursor from RGBA data.
    pub fn from_rgba(
        rgba: impl Into<Vec<u8>>,
        width: u16,
        height: u16,
        hotspot_x: u16,
        hotspot_y: u16,
    ) -> Cursor {
        let custom_cursor = CustomCursor {
            rgba: rgba.into(),
            width,
            height,
            hotspot_x,
            hotspot_y,
        };

        Cursor::Custom(custom_cursor)
    }

    /// Load custom cursor from image file.
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        hotspot_x: u16,
        hotspot_y: u16,
    ) -> Result<Cursor, image::ImageError> {
        let (rgba, width, height) = {
            let image = image::open(path)?.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width as u16, height as u16)
        };

        Ok(Cursor::from_rgba(rgba, width, height, hotspot_x, hotspot_y))
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::Icon(CursorIcon::default())
    }
}

#[derive(Debug, Clone)]
/// See [`winit::window::CustomCursor`].
pub struct CustomCursor {
    pub(crate) rgba: Vec<u8>,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) hotspot_x: u16,
    pub(crate) hotspot_y: u16,
}

impl From<CursorIcon> for Cursor {
    fn from(value: CursorIcon) -> Self {
        Cursor::Icon(value) 
    }
}

impl Cursor {
    pub fn into_winit_cursor(&self, event_loop: &winit::event_loop::ActiveEventLoop) -> winit::window::Cursor {
        match self.clone() {
            Self::Icon(icon) => winit::window::Cursor::Icon(icon),
            Self::Custom(custom) => {
                match winit::window::CustomCursor::from_rgba(
                    custom.rgba,
                    custom.width,
                    custom.height,
                    custom.hotspot_x,
                    custom.hotspot_y
                ) {
                    Ok(source) => event_loop.create_custom_cursor(source).into(),
                    Err(err) => {
                        eprintln!("Failed to create custom cursor: {}", err);
                        winit::window::Cursor::default()
                    }
                }

            }
        }
    }
}
