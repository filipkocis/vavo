//! Newtype wrappers for wgpu and other render types to improve type safety and abstraction.
use std::sync::Arc;

macro_rules! define_render_newtype {
    ($name:ident, $inner:ty, $doc:expr $(, $clone:ident)?) => {
        #[doc = $doc]
        #[derive(Debug, crate::macros::Resource)]
        pub struct $name($inner);

        impl $name {
            /// Creates a new wrapper around the inner value
            #[inline]
            pub(crate) fn new(inner: $inner) -> Self {
                Self(inner)
            }

            /// Consumes the wrapper and returns the inner value
            #[inline]
            pub(crate) fn unwrap(self) -> $inner {
                self.0
            }

            /// Replaces the inner value, returning the old one
            #[inline]
            pub(crate) fn replace(&mut self, inner: $inner) -> $inner {
                std::mem::replace(&mut self.0, inner)
            }

            // Expand clone method if specified
            define_render_newtype!(@maybe_clone $inner $(, $clone)?);
        }

        impl core::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    // clone helper arm
    (@maybe_clone $inner:ty) => {};
    (@maybe_clone $inner:ty, clone) => {
        /// Clones the wrapper
        #[inline]
        pub(crate) fn clone_wrapped(&self) -> Self
        where
            $inner: Clone,
        {
            Self(self.0.clone())
        }
    };
}

define_render_newtype!(
    RenderWindow,
    // has to be Arc to allow sharing between systems
    Arc<winit::window::Window>,
    "Newtype wrapper for [`winit::window::Window`].",
    clone
);

define_render_newtype!(
    RenderInstance,
    wgpu::Instance,
    "Newtype wrapper for [`wgpu::Instance`].",
    clone
);

define_render_newtype!(
    RenderSurface,
    // has to be Arc to allow sharing, even though it should only be used in one place
    Arc<wgpu::Surface<'static>>,
    "Newtype wrapper for [`wgpu::Surface`].",
    clone
);

define_render_newtype!(
    RenderSurfaceConfiguration,
    wgpu::SurfaceConfiguration,
    "Newtype wrapper for [`wgpu::SurfaceConfiguration`]. Mutations to the inner configuration will not affect the actual surface, this is purely for retrieving configuration data.",
    clone
);

define_render_newtype!(
    RenderAdapter,
    wgpu::Adapter,
    "Newtype wrapper for [`wgpu::Adapter`].",
    clone
);

define_render_newtype!(
    RenderDevice,
    wgpu::Device,
    "Newtype wrapper for [`wgpu::Device`].",
    clone
);

define_render_newtype!(
    RenderQueue,
    wgpu::Queue,
    "Newtype wrapper for [`wgpu::Queue`].",
    clone
);

//
//
//

define_render_newtype!(
    RenderSurfaceTexture,
    // has to be Arc because this is not a handle, but an owned structure
    Arc<wgpu::SurfaceTexture>,
    "Newtype wrapper for [`wgpu::SurfaceTexture`]."
);

/// "Newtype wrapper for [`wgpu::CommandEncoder`]."
#[derive(Debug)]
pub struct RenderCommandEncoder(wgpu::CommandEncoder);
impl RenderCommandEncoder {
    /// Creates a new wrapper around the inner value
    #[inline]
    pub(crate) fn new(device: &RenderDevice, label: &str) -> Self {
        let descriptor = wgpu::CommandEncoderDescriptor { label: Some(label) };
        let encoder = device.create_command_encoder(&descriptor);
        RenderCommandEncoder(encoder)
    }

    /// Consumes the wrapper and returns the inner value
    #[inline]
    pub(crate) fn unwrap(self) -> wgpu::CommandEncoder {
        self.0
    }
}

impl core::ops::Deref for RenderCommandEncoder {
    type Target = wgpu::CommandEncoder;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for RenderCommandEncoder {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Newtype wrapper for a queue of [`wgpu::CommandBuffer`]s.
#[derive(Debug)]
pub struct RenderCommandQueue(Vec<wgpu::CommandBuffer>);
impl RenderCommandQueue {
    /// Creates a new empty command queue
    #[inline]
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    /// Pushes a finished command encoder into the queue
    #[inline]
    pub(crate) fn push(&mut self, encoder: RenderCommandEncoder) {
        self.0.push(encoder.unwrap().finish());
    }

    /// Consumes the queue and returns the inner vector of command buffers
    #[inline]
    pub(crate) fn drain(&mut self) -> impl Iterator<Item = wgpu::CommandBuffer> {
        self.0.drain(..)
    }
}
