use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::*, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}
};

use crate::app::App;

use super::AppState;

pub struct AppHandler {
    app: App,
    state: Option<AppState>
}

impl AppHandler {
    pub fn init(app: App) -> (EventLoop<()>, Self) {
        let app = Self {
            app,
            state: None
        };

        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        (event_loop, app)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.state.as_mut().unwrap().resize(new_size);
        self.app.create_event("WindowResized"); // TODO
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes()
            .with_title("Game");

        let window = event_loop.create_window(window_attrs).unwrap();
        let mut state = AppState::new(window);

        if self.state.is_none() {
            self.app.startup(&mut state);
        }

        self.state = Some(state);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.app.create_event(DeviceEvent::MouseMotion { delta });
            },
            _ => (),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if id != self.state.as_ref().unwrap().window().id() {
            return
        }

        // TODO: handle events
        self.app.update(self.state.as_mut().unwrap());

        match event {
            WindowEvent::KeyboardInput { 
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Escape),  
                    ..
                },
                ..
            } |
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => self.resize(physical_size),
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.app.render(self.state.as_mut().unwrap()) {
                    match err {
                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                            eprintln!("Surface Lost or Outdated");
                            self.resize(*self.state.as_ref().unwrap().size());
                        },
                        wgpu::SurfaceError::OutOfMemory => {
                            eprintln!("Out Of Memory");
                            event_loop.exit();
                        },
                        wgpu::SurfaceError::Timeout => {
                            eprintln!("Surface Timeout");
                            self.state.as_mut().unwrap().reconfigure();
                        },
                    }
                }
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let window = self.state.as_ref().unwrap().window();
        window.request_redraw();
    }
}
