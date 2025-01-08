use crate::{core::graph::{CustomRenderGraphContext, RenderGraphContext}, query::Query, world::entities::Entities};

use super::SystemsContext;

pub struct System {
    name: String,
    func_ptr: *const (),
    exec: Box<dyn Fn(&mut SystemsContext, &mut Entities)>,
    conditions: Vec<Box<dyn Fn(&mut SystemsContext, &mut Entities) -> bool>>,
}

impl System {
    pub fn new<T: 'static, F: 'static>(name: &str, func: fn(&mut SystemsContext, Query<T, F>)) -> Self {
        Self {
            name: name.to_string(),
            func_ptr: func as *const (),
            exec: Box::new(move |ctx, entities| {
                let query = Query::new(entities);
                func(ctx, query);
            }),
            conditions: Vec::new(),
        }
    }

    pub(crate) fn run(&mut self, ctx: &mut SystemsContext, entities: &mut Entities) {
        if self.conditions.iter().all(|condition| condition(ctx, entities)) {
            (self.exec)(ctx, entities);
        }
    }

    /// Add new run condition to the system
    pub fn run_if<T: 'static, F: 'static>(mut self, condition: fn(&mut SystemsContext, Query<T, F>) -> bool) -> Self {
        self.conditions.push(Box::new(move |ctx, entities| -> bool {
            let query = Query::new(entities);
            condition(ctx, query)
        }));
        self
    }
}

pub struct GraphSystem {
    name: String,
    func_ptr: *const (),
    exec: Box<dyn FnMut(RenderGraphContext, &mut SystemsContext, &mut Entities)>,
}

impl GraphSystem {
    pub fn new<T: 'static, F: 'static>(name: &str, func: fn(RenderGraphContext, &mut SystemsContext, Query<T, F>)) -> Self {
        Self {
            name: name.to_string(),
            func_ptr: func as *const (),
            exec: Box::new(move |graph_ctx, ctx, entities| {
                let query = Query::new(entities);
                func(graph_ctx, ctx, query);
            }),
        }
    }

    pub(crate) fn run(&mut self, graph_ctx: RenderGraphContext, ctx: &mut SystemsContext, entities: &mut Entities) {
        (self.exec)(graph_ctx, ctx, entities);
    }
}

pub struct CustomGraphSystem {
    name: String,
    func_ptr: *const (),
    exec: Box<dyn FnMut(CustomRenderGraphContext, &mut SystemsContext, &mut Entities)>,
}

impl CustomGraphSystem {
    pub fn new<T: 'static, F: 'static>(name: &str, func: fn(CustomRenderGraphContext, &mut SystemsContext, Query<T, F>)) -> Self {
        Self {
            name: name.to_string(),
            func_ptr: func as *const (),
            exec: Box::new(move |graph_ctx, ctx, entities| {
                let query = Query::new(entities);
                func(graph_ctx, ctx, query);
            }),
        }
    }

    pub(crate) fn run(&mut self, graph_ctx: CustomRenderGraphContext, ctx: &mut SystemsContext, entities: &mut Entities) {
        (self.exec)(graph_ctx, ctx, entities);
    }
}
