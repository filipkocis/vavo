use crate::app::Plugin;
use crate::event::*;

/// Plugin for registering built-in event types from [`events`](crate::event::events)
pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut crate::prelude::App) {
        app.register_event::<DeviceEvent>()
            .register_event::<WindowEvent>()
            .register_event::<KeyboardInput>()
            .register_event::<MouseInput>()
            .register_event::<MouseWheel>()
            .register_event::<MouseMotion>()
            .register_event::<CursorMoved>();
    }
}
