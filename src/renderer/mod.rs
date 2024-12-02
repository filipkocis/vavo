mod material;
mod color;
mod image;
pub mod palette;

pub use material::Material;
pub use image::Image;
pub use color::Color;

pub enum Face {
    Front,
    Back,
}
