pub mod commands;
mod conflict;
mod into;
mod macros;
mod params;
mod scheduler;
mod tasks;

pub use commands::Commands;
use conflict::ConflictChecker;
pub use into::{IntoSystem, IntoSystemCondition};
pub use params::{ParamInfo, SystemParam, TypeInfo};
pub use scheduler::{
    label::{layer, phase},
    *,
};
pub use tasks::{AsyncTask, Task};

use crate::prelude::{Tick, World};

/// Per-system data passed to systems during execution.
/// Stores things like last run tick, system name, profiling info, etc.
pub struct SystemContext<'a> {
    pub last_run: &'a Tick,
    pub params_info: &'a [ParamInfo],
    pub exec_info: &'a TypeInfo,
}

impl<'a> SystemContext<'a> {
    /// Create new system context
    #[inline]
    pub fn new(last_run: &'a Tick, params_info: &'a [ParamInfo], exec_info: &'a TypeInfo) -> Self {
        Self {
            last_run,
            params_info,
            exec_info,
        }
    }
}

/// Function type for system execution
pub type SystemExecFn<Output> =
    dyn FnMut(&mut World, SystemContext) -> Output + Send + Sync + 'static;

/// System execution functions and its type information and `apply` function for post-processing
pub struct SystemExec<Output = ()> {
    /// Function's parameters info
    pub params_info: Vec<ParamInfo>,
    /// Function's type info
    pub exec_info: TypeInfo,
    /// System execution function
    exec: Box<SystemExecFn<Output>>,
    /// System init function
    init: Box<SystemExecFn<()>>,
    /// System apply function
    apply: Box<SystemExecFn<()>>,
}

impl<Output> SystemExec<Output> {
    /// Create a new system execution from function `exec` and its type information
    pub fn new(
        params_info: Vec<ParamInfo>,
        exec_info: TypeInfo,
        exec: Box<SystemExecFn<Output>>,
        init: Box<SystemExecFn<()>>,
        apply: Box<SystemExecFn<()>>,
    ) -> Self {
        Self {
            params_info,
            exec_info,
            exec,
            init,
            apply,
        }
    }

    /// Execute the system function
    #[inline]
    pub fn run(&mut self, world: &mut World, last_run: &Tick) -> Output {
        let context = SystemContext::new(last_run, &self.params_info, &self.exec_info);
        (self.exec)(world, context)
    }

    /// Execute the system's init function
    #[inline]
    pub fn init(&mut self, world: &mut World, last_run: &Tick) {
        let context = SystemContext::new(last_run, &self.params_info, &self.exec_info);
        (self.init)(world, context);
    }

    /// Execute the system's apply function
    #[inline]
    pub fn apply(&mut self, world: &mut World, last_run: &Tick) {
        let context = SystemContext::new(last_run, &self.params_info, &self.exec_info);
        (self.apply)(world, context);
    }
}

/// A system to be executed in the ECS world
pub struct System {
    /// Tick of the last run, or `0`
    last_run: Tick,
    /// System execution
    pub(super) exec: SystemExec,
    /// Run conditions
    pub(super) conditions: Vec<SystemCondition>,
}

impl System {
    /// Same as [`IntoSystem::run_if`] but internal to avoid the need for generic parameters
    #[inline]
    fn internal_run_if(mut self, condition: SystemCondition) -> System {
        self.conditions.push(condition);
        self
    }

    /// Execute system if all conditions are met
    pub fn run(&mut self, world: &mut World) {
        // TODO: handle world tick overflow
        if self.satisfies_conditions(world) {
            // Increment must come first to ensure `system.last_run < world.tick`
            world.tick.increment();
            self.exec.run(world, &self.last_run);
            self.last_run = *world.tick;
        }
    }

    /// Initializes the system.
    #[inline]
    pub fn init(&mut self, world: &mut World) {
        self.conditions.iter_mut().for_each(|c| c.init(world));
        self.exec.init(world, &self.last_run);
    }

    /// Applies any changes to the world needed after system execution by system parameters.
    /// This is run on the main thread after all parallel systems have finished executing.
    #[inline]
    pub fn apply(&mut self, world: &mut World) {
        self.conditions.iter_mut().for_each(|c| c.apply(world));
        self.exec.apply(world, &self.last_run);
    }

    /// Check if all run conditions are satisfied
    #[inline]
    fn satisfies_conditions(&mut self, world: &mut World) -> bool {
        self.conditions
            .iter_mut()
            .all(|condition| condition.run(world))
    }
}

/// A condition to be checked before running a [`System`]
pub struct SystemCondition {
    /// Tick of the last run, or `0`
    last_run: Tick,
    /// Condition execution
    pub(super) exec: SystemExec<bool>,
}

impl SystemCondition {
    /// Execute the condition function
    #[inline]
    pub fn run(&mut self, world: &mut World) -> bool {
        world.tick.increment();
        let result = self.exec.run(world, &self.last_run);
        self.last_run = *world.tick;
        result
    }

    /// Initializes the condition.
    #[inline]
    pub fn init(&mut self, world: &mut World) {
        self.exec.init(world, &self.last_run);
    }

    /// Applies any changes to the world needed after condition execution by system parameters.
    #[inline]
    pub fn apply(&mut self, world: &mut World) {
        self.exec.apply(world, &self.last_run);
    }
}
