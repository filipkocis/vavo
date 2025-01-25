use std::path::Path;

#[derive(Default, Debug, Clone)]
/// See `window_icon` as defined in [`winit::window::WindowAttributes`]
pub enum Icon {
    #[default]
    None,
    Icon(CustomIcon),
}

#[derive(Debug, Clone)]
/// See [`winit::window::Icon`].
pub struct CustomIcon {
    pub(crate) rgba: Vec<u8>,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

impl Icon {
    /// Load custom cursor from RGBA data.
    pub fn from_rgba(
        rgba: impl Into<Vec<u8>>,
        width: u16,
        height: u16,
    ) -> Self {
        let custom_icon = CustomIcon {
            rgba: rgba.into(),
            width,
            height,
        };

        Self::Icon(custom_icon)
    }

    /// Load custom cursor from image file.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, image::ImageError> {
        let (rgba, width, height) = {
            let image = image::open(path)?.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width as u16, height as u16)
        };

        Ok(Self::from_rgba(rgba, width, height))
    }
}
