mod material;
mod color;
mod image;
pub mod palette;
mod mesh;
pub mod culling;

pub use material::Material;
pub use image::{Texture, Image, SingleColorTexture};
pub use color::Color;
pub use mesh::{Mesh, Meshable};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Front,
    #[default]
    Back,
}
