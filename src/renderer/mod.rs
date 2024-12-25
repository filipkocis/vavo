mod material;
mod color;
mod image;
pub mod palette;
mod mesh;
pub mod shapes;
pub mod sphere;

pub use material::Material;
pub use image::{Texture, Image, SingleColorTexture};
pub use color::Color;
pub use mesh::{Mesh, Meshable};

pub enum Face {
    Front,
    Back,
}
