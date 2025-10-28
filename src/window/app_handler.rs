use glam::Vec2;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowId,
};

use crate::{
    app::App,
    event::events::{CursorMoved, MouseMotion, MouseWheel},
};

use super::{AppState, config::WindowConfig};

pub struct AppHandler<'a> {
    app: &'a mut App,
    state: Option<AppState>,
}

impl<'a> AppHandler<'a> {
    pub fn init(app: &'a mut App) -> (EventLoop<()>, Self) {
        let app = Self { app, state: None };

        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        (event_loop, app)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.state
            .as_mut()
            .unwrap()
            .resize(new_size, &mut self.app.world.resources);
        self.app.resize(new_size);
    }
}

impl<'a> ApplicationHandler for AppHandler<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_config = self.app.world.resources.try_get::<WindowConfig>();

        let window_attrs = match window_config {
            Some(ref config) => config.get_window_attributes(),
            None => WindowConfig::default().get_window_attributes(),
        };

        let window = event_loop.create_window(window_attrs).unwrap();

        if let Some(config) = window_config {
            config.post_apply(&window, event_loop);
        }

        let state = AppState::new(window);
        state.apply_to_resources(&mut self.app.world.resources);

        if self.state.is_none() {
            self.app.startup();
        }

        self.state = Some(state);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.app.create_event(event.clone());

        match event {
            DeviceEvent::MouseMotion { delta } => {
                let delta = Vec2::new(delta.0 as f32, delta.1 as f32);
                self.app.create_event(MouseMotion { delta })
            }
            _ => (),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if id != self.state.as_ref().unwrap().window().id() {
            return;
        }

        self.app.create_event(event.clone());

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            }
            | WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput { event, .. } => {
                self.app.handle_keyboard_input(event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.app.handle_mouse_input(state, button);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.app.create_event(MouseWheel { delta });
            }
            WindowEvent::CursorMoved { position, .. } => {
                let position = Vec2::new(position.x as f32, position.y as f32);
                self.app.create_event(CursorMoved { position });
                self.state
                    .as_mut()
                    .unwrap()
                    .update_cursor_position(Some(position), &mut self.app.world.resources);
            }

            WindowEvent::Resized(physical_size) => self.resize(physical_size),
            WindowEvent::RedrawRequested => {
                self.app.update();

                if let Err(err) = self.app.render() {
                    match err {
                        wgpu::SurfaceError::Lost
                        | wgpu::SurfaceError::Outdated
                        | wgpu::SurfaceError::Other => {
                            eprintln!("Surface Lost or Outdated");
                            self.state.as_ref().unwrap().reconfigure();
                        }
                        wgpu::SurfaceError::OutOfMemory => {
                            eprintln!("Out Of Memory");
                            event_loop.exit();
                        }
                        wgpu::SurfaceError::Timeout => {
                            eprintln!("Surface Timeout");
                            self.state.as_ref().unwrap().reconfigure();
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.state.as_ref().unwrap().window().request_redraw();
    }
}
