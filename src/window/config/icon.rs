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
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Icon {
    /// Load custom cursor from RGBA data.
    pub fn from_rgba(
        rgba: impl Into<Vec<u8>>,
        width: u32,
        height: u32,
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
            (rgba, width, height)
        };

        Ok(Self::from_rgba(rgba, width, height))
    }
}

impl From<Icon> for Option<winit::window::Icon> {
    fn from(value: Icon) -> Self {
        match value {
            Icon::None => None,
            Icon::Icon(ico) => {
                match winit::window::Icon::from_rgba(ico.rgba, ico.width, ico.height) {
                    Ok(icon) => Some(icon),
                    Err(err) => {
                        eprintln!("Failed to window create icon: {}", err);
                        None
                    }
                }
            }
        } 
    }
}
