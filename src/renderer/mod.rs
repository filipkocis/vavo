mod color;
pub mod culling;
mod image;
mod material;
mod mesh;
pub mod newtype;
pub mod palette;

pub use color::Color;
pub use image::{Image, SingleColorTexture, Texture};
pub use material::Material;
pub use mesh::{Mesh, Meshable};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Front,
    #[default]
    Back,
}
