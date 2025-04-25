use std::any::TypeId;

use crate::{core::graph::{CustomRenderGraphContext, RenderGraphContext}, ecs::entities::Entities, prelude::Tick, query::Query};

use super::{IntoSystemCondition, SystemsContext};

pub struct System {
    /// Type name of the system's `exec` function
    type_name: &'static str,
    /// TypeID of the system's `exec` function
    type_id: TypeId,
    /// Tick of the last run, or `0` 
    last_run: Tick,
    exec: Box<dyn Fn(&mut SystemsContext, &mut Entities)>,
    conditions: Vec<Box<dyn Fn(&mut SystemsContext, &mut Entities) -> bool>>,
}

impl System {
    /// Create a new system from function `exec` where `type_name` and `type_id` should be derived from
    /// the function type.
    ///
    /// # Note
    /// Use trait [`IntoSystem`] instead of this function.
    pub fn new<E, T, F>(exec: E, type_name: &'static str, type_id: TypeId) -> Self 
    where 
        E: Fn(&mut SystemsContext, Query<T, F>) + 'static 
    {
        Self {
            type_name,
            type_id,
            last_run: Tick::default(),
            exec: Box::new(move |ctx, entities| {
                let query = Query::new(entities);
                exec(ctx, query);
            }),
            conditions: Vec::new(),
        }
    }

    /// Returns function's type name
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Execute system if all conditions are met
    pub(crate) fn run(&mut self, ctx: &mut SystemsContext, entities: &mut Entities) {
        if self.conditions.iter().all(|condition| condition(ctx, entities)) {
            // Increment must come first to ensure `sys.last_run < world.tick`
            ctx.increment_world_tick();
            (self.exec)(ctx, entities);
            self.last_run = ctx.world_tick();
        }
    }

    /// Add new run condition to the system
    pub fn run_if<T: 'static, F: 'static>(mut self, condition: impl IntoSystemCondition<T, F>) -> Self {
        let condition = condition.system_condition();
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
    /// Tick of the last run, or `0` 
    last_run: Tick,
    exec: Box<dyn FnMut(RenderGraphContext, &mut SystemsContext, &mut Entities)>,
}

impl GraphSystem {
    pub fn new<T: 'static, F: 'static>(name: &str, func: fn(RenderGraphContext, &mut SystemsContext, Query<T, F>)) -> Self {
        Self {
            name: name.to_string(),
            func_ptr: func as *const (),
            last_run: Tick::default(),
            exec: Box::new(move |graph_ctx, ctx, entities| {
                let query = Query::new(entities);
                func(graph_ctx, ctx, query);
            }),
        }
    }

    pub(crate) fn run(&mut self, graph_ctx: RenderGraphContext, ctx: &mut SystemsContext, entities: &mut Entities) {
        ctx.increment_world_tick();
        (self.exec)(graph_ctx, ctx, entities);
        self.last_run = ctx.world_tick();
    }
}

pub struct CustomGraphSystem {
    name: String,
    func_ptr: *const (),
    /// Tick of the last run, or `0` 
    last_run: Tick,
    exec: Box<dyn FnMut(CustomRenderGraphContext, &mut SystemsContext, &mut Entities)>,
}

impl CustomGraphSystem {
    pub fn new<T: 'static, F: 'static>(name: &str, func: fn(CustomRenderGraphContext, &mut SystemsContext, Query<T, F>)) -> Self {
        Self {
            name: name.to_string(),
            func_ptr: func as *const (),
            last_run: Tick::default(),
            exec: Box::new(move |graph_ctx, ctx, entities| {
                let query = Query::new(entities);
                func(graph_ctx, ctx, query);
            }),
        }
    }

    pub(crate) fn run(&mut self, graph_ctx: CustomRenderGraphContext, ctx: &mut SystemsContext, entities: &mut Entities) {
        ctx.increment_world_tick();
        (self.exec)(graph_ctx, ctx, entities);
        self.last_run = ctx.world_tick();
    }
}
