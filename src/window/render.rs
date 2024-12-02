use std::{ops::{Deref, DerefMut}, sync::Arc};

use winit::{dpi::PhysicalSize, window::Window};

use super::AppState;

pub struct Renderer<'a>(*mut RenderContext<'a>);

impl<'a> Deref for Renderer<'a> {
    type Target = RenderContext<'a>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<'a> DerefMut for Renderer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

pub struct RenderContext<'a> {
    target: Option<(wgpu::SurfaceTexture, wgpu::TextureView)>,
    encoder: Option<*mut wgpu::CommandEncoder>,
    state: &'a mut AppState,
}

impl<'a> Drop for RenderContext<'a> {
    fn drop(&mut self) {
        let encoder = unsafe { Box::from_raw(self.encoder.take().unwrap()) };
        self.state.queue.submit(Some(encoder.finish()));
        self.target.take().map(|(st, _)| st.present());
    }
}

impl<'a> RenderContext<'a> {
    pub(crate) fn new(state: &'a mut AppState) -> Result<Self, wgpu::SurfaceError> {
        let target = Some(Self::create_target(&state.surface)?);
        let encoder = Some(Self::create_encoder(&state.device));

        Ok(Self {
            target,
            encoder,
            state,
        })
    }

    pub(crate) fn as_renderer(&mut self) -> Renderer<'a> {
        Renderer(self)
    } 

    fn create_target(surface: &wgpu::Surface) -> Result<
        (wgpu::SurfaceTexture, wgpu::TextureView),
        wgpu::SurfaceError
    > {
        let surface_texture = surface.get_current_texture()?;
        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok((surface_texture, texture_view))
    }

    fn create_encoder(device: &wgpu::Device) -> *mut wgpu::CommandEncoder {
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        Box::into_raw(Box::new(encoder))
    }
}

pub struct CommandEncoder<'a> {
    inner: *mut wgpu::CommandEncoder,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl CommandEncoder<'_> {
    pub fn new(encoder: *mut wgpu::CommandEncoder) -> Self {
        Self {
            inner: encoder,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Deref for CommandEncoder<'_> {
    type Target = wgpu::CommandEncoder;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl DerefMut for CommandEncoder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

#[allow(dead_code)]
impl RenderContext<'_> {
    pub(crate) fn surface(&self) -> &wgpu::Surface {
        &self.state.surface
    }

    pub fn target(&self) -> (&wgpu::SurfaceTexture, &wgpu::TextureView) {
        let target = self.target.as_ref().unwrap();
        (&target.0, &target.1)
    }

    pub fn view(&self) -> &wgpu::TextureView {
        let target = self.target.as_ref().unwrap();
        &target.1
    }

    pub fn encoder(&self) -> CommandEncoder {
        CommandEncoder::new(self.encoder.unwrap())
    }
 
    pub fn device(&self) -> &wgpu::Device {
        &self.state.device
    }

    pub(crate) fn queue(&self) -> &wgpu::Queue {
        &self.state.queue
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.state.config
    }

    pub fn size(&self) -> &PhysicalSize<u32> {
        &self.state.size
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.state.window
    }
}
