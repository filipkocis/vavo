use crate::{app::{EventReader, EventWriter, Events}, window::RenderContext, world::resources::Resources};

use super::Commands;


pub struct SystemsContext<'a, 'b> {
    pub commands: Commands,
    pub resources: &'a mut Resources,
    pub event_writer: EventWriter<'a>,
    pub event_reader: EventReader<'a>,
    pub renderer: &'a mut RenderContext<'b>,
}

impl<'a, 'b> SystemsContext<'a, 'b> {
    pub fn new(commands: Commands, resources: &'a mut Resources, events: &'a mut Events, renderer: &'a mut RenderContext<'b>) -> Self {
        let (event_reader, event_writer) = events.handlers();

        Self {
            commands,
            resources,
            event_writer, 
            event_reader, 
            renderer,
        }
    }
}
