use std::sync::Arc;

use glam::Vec2;
use pollster::FutureExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{prelude::Resources, renderer::newtype::*};

/// Holds Window - GPU state for the application. Used by the AppHandler
pub(crate) struct AppState {
    instance: RenderInstance,
    surface: Option<RenderSurface>,
    window: RenderWindow,
    adapter: RenderAdapter,
    device: RenderDevice,
    queue: RenderQueue,
    config: RenderSurfaceConfiguration,

    size: PhysicalSize<u32>,
    cursor_position: Option<Vec2>,
}

impl AppState {
    /// Create new AppState from a winit window.
    /// You should call `apply_to_resources` to sync with ECS resources.
    pub fn new(window: Window) -> Self {
        let instance = Self::create_gpu_instance();
        let window = Arc::new(window);

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = Self::create_adapter(&instance, &surface);
        let (device, queue) = Self::create_device(&adapter);
        let surface_caps = surface.get_capabilities(&adapter);

        let size = window.inner_size();
        let config = Self::create_surface_config(surface_caps, size);
        surface.configure(&device, &config);

        // Wrap in shareable newtypes, second clone of these will be in Resources
        let instance = RenderInstance::new(instance);
        let surface = RenderSurface::new(surface);
        let window = RenderWindow::new(window);
        let adapter = RenderAdapter::new(adapter);
        let device = RenderDevice::new(device);
        let queue = RenderQueue::new(queue);
        let config = RenderSurfaceConfiguration::new(config);

        Self {
            instance,
            surface: Some(surface),
            window,
            adapter,
            device,
            queue,
            config,

            size,
            cursor_position: None,
        }
    }

    /// Insert all GPU resources into ECS resources
    pub fn apply_to_resources(&mut self, resources: &mut Resources) {
        resources.insert(self.instance.clone_wrapped());
        resources.insert(self.surface.take().unwrap());
        resources.insert(self.window.clone_wrapped());
        resources.insert(self.adapter.clone_wrapped());
        resources.insert(self.device.clone_wrapped());
        resources.insert(self.queue.clone_wrapped());
        resources.insert(self.config.clone_wrapped());

        let mut window = crate::prelude::Window::default();
        window.size = self.size;
        window.cursor_position = self.cursor_position;
        resources.insert(window);
    }

    /// Get a reference to the winit window
    #[inline]
    pub(crate) fn window(&self) -> &RenderWindow {
        &self.window
    }

    /// Resize the surface and reconfigue it
    #[inline]
    pub fn resize(&mut self, new_size: PhysicalSize<u32>, resources: &mut Resources) {
        self.size = new_size;

        self.config.width = new_size.width;
        self.config.height = new_size.height;

        let mut window = resources.get_mut::<crate::prelude::Window>();
        window.size = new_size;
        let mut config = resources.get_mut::<RenderSurfaceConfiguration>();
        config.width = new_size.width;
        config.height = new_size.height;

        self.reconfigure(resources);
    }

    /// Update the cursor position
    #[inline]
    pub fn update_cursor_position(&mut self, position: Option<Vec2>, resources: &mut Resources) {
        self.cursor_position = position;

        let mut window = resources.get_mut::<crate::prelude::Window>();
        window.cursor_position = position;
    }

    /// Reconfigure the surface with the current config
    #[inline]
    pub fn reconfigure(&self, resources: &mut Resources) {
        let surface = resources.get::<RenderSurface>();
        surface.configure(&self.device, &self.config);
    }

    #[inline]
    fn create_gpu_instance() -> wgpu::Instance {
        wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        })
    }

    #[inline]
    fn create_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        };

        instance
            .request_adapter(&adapter_options)
            .block_on()
            .expect("Failed to create adapter")
    }

    #[inline]
    fn create_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        let device_descriptor = wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::PUSH_CONSTANTS,
            required_limits: wgpu::Limits {
                max_push_constant_size: 128,
                ..wgpu::Limits::default()
            },
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        };

        adapter
            .request_device(&device_descriptor)
            .block_on()
            .expect("Failed to create device")
    }

    #[inline]
    fn create_surface_config(
        capabilities: wgpu::SurfaceCapabilities,
        size: PhysicalSize<u32>,
    ) -> wgpu::SurfaceConfiguration {
        let surface_format = capabilities
            .formats
            .iter()
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
