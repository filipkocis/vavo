use std::fmt::Display;

use crate::{assets::Handle, prelude::Image};

/// Describes the color target view to use in a render pass node
pub enum NodeColorTarget {
    /// Will use the swapchain
    Surface,
    /// Useful for shadow maps
    None,
    /// Use a texture from the asset manager as the color target
    ///
    /// # Note
    /// Handle has to be valid unless regenerated manually
    Image(Handle<Image>),
    /// Use an owned custom image as the color target
    Owned(Image),
    /// Use the output of another node as the target
    ///
    /// # Note
    /// If you need the node to run first, add it as a dependency
    ///
    /// # Panics
    /// Panics if the node does not exist
    Node(String),
}

/// Describes the depth target view to use in a render pass node
pub enum NodeDepthTarget {
    /// No depth target will be used
    None,
    /// Use a texture from the asset manager as the depth target
    ///
    /// # Note
    /// Handle has to be valid unless regenerated manually
    Image(Handle<Image>),
    /// Use an owned custom image as the depth target
    Owned(Image),
    /// Use the output of another node as the target
    ///
    /// # Note
    /// If you need the node to run first, add it as a dependency
    ///
    /// # Panics
    /// Panics if the node does not exist
    Node(String),
}

impl Display for NodeColorTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeColorTarget::Surface => write!(f, "NodeColorTarget::Surface"),
            NodeColorTarget::None => write!(f, "NodeColorTarget::None"),
            NodeColorTarget::Image(_) => write!(f, "NodeColorTarget::Image(..)"),
            NodeColorTarget::Owned(_) => write!(f, "NodeColorTarget::Owned(..)"),
            NodeColorTarget::Node(name) => write!(f, "NodeColorTarget::Node({})", name),
        }
    }
}

impl Display for NodeDepthTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeDepthTarget::None => write!(f, "NodeDepthTarget::None"),
            NodeDepthTarget::Image(_) => write!(f, "NodeDepthTarget::Image(..)"),
            NodeDepthTarget::Owned(_) => write!(f, "NodeDepthTarget::Owned(..)"),
            NodeDepthTarget::Node(name) => write!(f, "NodeDepthTarget::Node({})", name),
        }
    }
}
