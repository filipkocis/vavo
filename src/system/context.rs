use crate::{app::{EventReader, EventWriter, Events}, core::graph::RenderGraph, resources::Resources, window::Renderer, world::World};

use super::Commands;


pub struct SystemsContext<'a, 'b> {
    pub commands: Commands,
    pub resources: &'a mut Resources,
    pub event_writer: EventWriter<'a>,
    pub event_reader: EventReader<'a>,
    pub renderer: Renderer<'b>,
    /// Raw pointer to the world, should not be used unless you know what you're doing.
    /// SAFETY: This is always a valid pointer to a World instance.
    pub world: *mut World,
    /// Unsafe raw pointer if inside a node system, should not be used or modified unless you are sure what
    /// you are doing.
    ///
    /// # Note
    /// It should be used only inside startup systems to edit nodes in the graph.
    pub graph: *mut RenderGraph,
}

impl<'a, 'b> SystemsContext<'a, 'b> {
    pub fn new(commands: Commands, resources: &'a mut Resources, events: &'a mut Events, renderer: Renderer<'b>, world: *mut World, graph: *mut RenderGraph) -> Self {
        let (event_reader, event_writer) = events.handlers();

        Self {
            commands,
            resources,
            event_writer, 
            event_reader, 
            renderer,
            world,
            graph,
        }
    }
}
