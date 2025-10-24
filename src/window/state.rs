use std::sync::Arc;

use glam::Vec2;
use pollster::FutureExt;
use winit::{dpi::PhysicalSize, window::Window};

pub struct AppState {
    pub(super) surface: wgpu::Surface<'static>,
    pub(super) device: wgpu::Device,
    pub(super) config: wgpu::SurfaceConfiguration,
    pub(super) size: PhysicalSize<u32>,
    pub(super) queue: wgpu::Queue,
    pub(super) window: Arc<Window>,
    pub(super) cursor_position: Option<Vec2>,
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
            cursor_position: None,
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
        wgpu::Instance::new(&wgpu::InstanceDescriptor {
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

        instance.request_adapter(&adapter_options).block_on().expect("Failed to create adapter")
    }

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

        adapter.request_device(&device_descriptor)
            .block_on()
            .expect("Failed to create device")
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
