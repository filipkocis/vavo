//! Newtype wrappers for wgpu and other render types to improve type safety and abstraction.
use std::sync::Arc;

macro_rules! define_render_newtype {
    ($name:ident, $inner:ty, $doc:expr) => {
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

            /// Clones the wrapper
            #[inline]
            pub(crate) fn clone_wrapped(&self) -> Self {
                Self(self.0.clone())
            }
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
}

define_render_newtype!(
    RenderWindow,
    // has to be Arc to allow sharing between systems
    Arc<winit::window::Window>,
    "Newtype wrapper for [`winit::window::Window`]."
);

define_render_newtype!(
    RenderInstance,
    wgpu::Instance,
    "Newtype wrapper for [`wgpu::Instance`]."
);

define_render_newtype!(
    RenderSurface,
    // has to be Arc to allow sharing, even though it should only be used in one place
    Arc<wgpu::Surface<'static>>,
    "Newtype wrapper for [`wgpu::Surface`]."
);

define_render_newtype!(
    RenderSurfaceConfiguration,
    wgpu::SurfaceConfiguration,
    "Newtype wrapper for [`wgpu::SurfaceConfiguration`]. Mutations to the inner configuration will not affect the actual surface, this is purely for retrieving configuration data."
);

define_render_newtype!(
    RenderAdapter,
    wgpu::Adapter,
    "Newtype wrapper for [`wgpu::Adapter`]."
);

define_render_newtype!(
    RenderDevice,
    wgpu::Device,
    "Newtype wrapper for [`wgpu::Device`]."
);

define_render_newtype!(
    RenderQueue,
    wgpu::Queue,
    "Newtype wrapper for [`wgpu::Queue`]."
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

define_render_newtype!(
    RenderCommandEncoder,
    // has to be Arc to allow sharing between render systems
    Arc<wgpu::CommandEncoder>,
    "Newtype wrapper for [`wgpu::CommandEncoder`]."
);
