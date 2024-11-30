use crate::{entities::Entities, events::{EventReader, EventWriter, Events}, prelude::Commands, query::Query, resources::Resources};

pub struct SystemsContext<'a> {
    pub commands: Commands,
    pub resources: &'a mut Resources,
    pub event_writer: EventWriter<'a>,
    pub event_reader: EventReader<'a>,
}

impl<'a> SystemsContext<'a> {
    pub fn new(commands: Commands, resources: &'a mut Resources, events: &'a mut Events) -> Self {
        let (event_reader, event_writer) = events.handlers();

        Self {
            commands,
            resources,
            event_writer, 
            event_reader, 
        }
    }
}

pub struct System {
    name: String,
    func_ptr: *const (),
    exec: Box<dyn FnMut(&mut SystemsContext, &mut Entities)>,
}

impl System {
    pub fn new<T: 'static>(name: &str, func: fn(&mut SystemsContext, Query<T>)) -> System {
        System {
            name: name.to_string(),
            func_ptr: func as *const (),
            exec: Box::new(move |ctx, entities| {
                let query = Query::new(entities);
                func(ctx, query);
            }),
        }
    }

    pub fn run(&mut self, ctx: &mut SystemsContext, entities: &mut Entities) {
        (self.exec)(ctx, entities);
    }
}
