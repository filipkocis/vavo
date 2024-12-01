use std::sync::Arc;

use pollster::FutureExt;
use wgpu::SurfaceError;
use winit::{dpi::PhysicalSize, window::Window};

pub struct AppState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    queue: wgpu::Queue,
    window: Arc<Window>,
}

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
        SurfaceError
    > {
        let surface_texture = surface.get_current_texture()?;
        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok((surface_texture, texture_view))
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

impl AppState {
    pub fn new(window: Window) -> Self {
        let instance = Self::create_gpu_instance();
        let window = Arc::new(window);

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = Self::create_adapter(instance, &surface);
        let (device, queue) = Self::create_device(&adapter);
        let surface_caps = surface.get_capabilities(&adapter);

        let size = window.inner_size();
        let config = Self::create_surface_config(surface_caps, size);
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
        }
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn size(&self) -> &PhysicalSize<u32> {
        &self.size
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;

        self.config.width = new_size.width;
        self.config.height = new_size.height;

        self.reconfigure();
    }

    pub fn reconfigure(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }

    fn create_gpu_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        })
    }

    fn create_adapter(instance: wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };

        instance.request_adapter(&adapter_options).block_on().unwrap()
    }

    fn create_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
            memory_hints: Default::default(),
        };

        adapter.request_device(&device_descriptor, None)
            .block_on()
            .unwrap()
    }

    fn create_surface_config(capabilities: wgpu::SurfaceCapabilities, size: PhysicalSize<u32>) -> wgpu::SurfaceConfiguration {
        let surface_format = capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            // view_formats: vec![surface_format.add_srgb_suffix()],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }
}
