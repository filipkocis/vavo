use crate::{core::graph::{CustomRenderGraphContext, RenderGraphContext}, query::Query, world::entities::Entities};

use super::SystemsContext;

pub struct System {
    name: String,
    func_ptr: *const (),
    exec: Box<dyn Fn(&mut SystemsContext, &mut Entities)>,
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
        }
    }

    pub(crate) fn run(&mut self, ctx: &mut SystemsContext, entities: &mut Entities) {
        (self.exec)(ctx, entities);
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
