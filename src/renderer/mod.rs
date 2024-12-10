mod material;
mod color;
mod image;
pub mod palette;
mod mesh;
pub mod shapes;

pub use material::Material;
pub use image::{Texture, Image, DefaultTexture};
pub use color::Color;
pub use mesh::{Mesh, Meshable};

pub enum Face {
    Front,
    Back,
}
