use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use super::AppState;


pub struct RenderContext<'a> {
    target: Option<(wgpu::SurfaceTexture, wgpu::TextureView)>,
    state: &'a mut AppState,
}

impl<'a> Drop for RenderContext<'a> {
    fn drop(&mut self) {
        self.target.take().map(|(st, _)| st.present());
    }
}

impl<'a> RenderContext<'a> {
    pub fn new(state: &'a mut AppState) -> Result<Self, wgpu::SurfaceError> {
        let target = Some(Self::create_target(&state.surface)?);

        Ok(Self {
            target,
            state,
        })
    }

    fn create_target(surface: &wgpu::Surface) -> Result<
        (wgpu::SurfaceTexture, wgpu::TextureView),
        wgpu::SurfaceError
    > {
        let surface_texture = surface.get_current_texture()?;
        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok((surface_texture, texture_view))
    }
}

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

    pub(crate) fn surface_mut(&mut self) -> &mut wgpu::Surface<'static> {
        &mut self.state.surface
    }

    pub fn target_mut(&mut self) -> (&mut wgpu::SurfaceTexture, &mut wgpu::TextureView) {
        let target = self.target.as_mut().unwrap();
        (&mut target.0, &mut target.1)
    }

    pub fn config_mut(&mut self) -> &mut wgpu::SurfaceConfiguration {
        &mut self.state.config
    }

    pub fn size_mut(&mut self) -> &mut PhysicalSize<u32> {
        &mut self.state.size
    }

    pub fn window_mut(&mut self) -> &mut Arc<Window> {
        &mut self.state.window
    }
}
