use crate::{
    app::App,
    core::graph::RenderGraph,
    ecs::resources::Resources,
    event::{
        event_handler::{EventReader, EventWriter},
        Events,
    },
    prelude::Tick,
    window::Renderer,
};

use super::Commands;

pub struct SystemsContext<'a, 'b, 'c> {
    pub commands: Commands<'c, 'c>,
    pub resources: &'a mut Resources,
    pub event_writer: EventWriter<'a>,
    pub event_reader: EventReader<'a>,
    pub renderer: Renderer<'b>,
    /// Raw pointer to the App, should not be used unless you know what you're doing.
    /// SAFETY: This is always a valid pointer to an App instance.
    pub app: *mut App,
    /// Unsafe raw pointer if inside a node system, should not be used or modified unless you are sure what
    /// you are doing.
    ///
    /// # Note
    /// It should be used only inside startup systems to edit nodes in the graph.
    pub graph: *mut RenderGraph,
}

impl<'a, 'b, 'c> SystemsContext<'a, 'b, 'c> {
    #[inline]
    pub fn new(
        commands: Commands<'c, 'c>,
        resources: &'a mut Resources,
        events: &'a mut Events,
        renderer: Renderer<'b>,
        app: *mut App,
        graph: *mut RenderGraph,
    ) -> Self {
        let (event_reader, event_writer) = events.handlers();

        Self {
            commands,
            resources,
            event_writer,
            event_reader,
            renderer,
            app,
            graph,
        }
    }

    /// Returns the current world tick
    #[inline]
    pub fn world_tick(&self) -> Tick {
        *unsafe { &*self.app }.world.tick
    }

    /// Increments the world tick
    #[inline]
    pub(crate) fn increment_world_tick(&mut self) {
        unsafe { &mut *self.app }.world.tick.increment();
    }

    /// Flushes stored commands and applies them to the world. Commands get flushed automatically
    /// after each system stage has finished, this can be used to flush them manually.
    ///
    /// # Safety
    /// This function mutably borrows the App's world, so the caller must ensure that no other
    /// mutable references exist to the world.
    pub unsafe fn flush_internal_commands(&mut self) {
        let app = unsafe { &mut *self.app };
        self.commands.apply(&mut app.world)
    }
}
