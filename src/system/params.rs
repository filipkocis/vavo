use crate::{
    app::App,
    core::graph::RenderGraph,
    event::event_handler::{EventReader, EventWriter},
    prelude::{Component, EntityId, Mut, Ref, Res, ResMut, Resource, World},
    query::{Query, filter::QueryFilter},
    renderer::newtype::{RenderCommandEncoder, RenderDevice},
    system::{Commands, SystemContext, commands::CommandQueue},
};
use std::any::{TypeId, type_name};

/// Type information for system functions and parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeInfo {
    name: &'static str,
    id: TypeId,
}

impl TypeInfo {
    /// Create new type information
    #[inline]
    pub fn new(name: &'static str, id: TypeId) -> Self {
        Self { name, id }
    }

    /// Returns the type name
    #[inline]
    pub fn type_name(&self) -> &'static str {
        self.name
    }

    /// Returns the type id
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.id
    }
}

/// Access information for system function parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamInfo {
    is_mutable: bool,
    type_info: TypeInfo,
}

impl ParamInfo {
    /// Create new parameter information
    #[inline]
    pub fn new(is_mutable: bool, type_info: TypeInfo) -> Self {
        Self {
            is_mutable,
            type_info,
        }
    }

    /// Returns `true` if the access is mutable
    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.is_mutable
    }

    /// Returns the parameter's type information
    #[inline]
    pub fn type_info(&self) -> TypeInfo {
        self.type_info
    }
}

/// Trait for types that can provide information about their parameter types and access patterns.
pub trait IntoParamInfo {
    /// Returns information about the parameter types and their access patterns.
    fn params_info() -> Vec<ParamInfo>;
}

/// Macro to implement [`IntoParamInfo`] for single types
macro_rules! impl_into_param_info {
    ($type:ident $(: $trait:ident)?, $for:ty, $is_mutable:expr) => {
        impl$(<$type : $trait>)? IntoParamInfo for $for {
            fn params_info() -> Vec<ParamInfo> {
                vec![ParamInfo::new(
                    $is_mutable,
                    TypeInfo::new(type_name::<$type>(), TypeId::of::<$type>()),
                )]
            }
        }
    };
}

// Special app params
impl_into_param_info!(App, &mut App, true);
impl_into_param_info!(World, &mut World, true);
impl_into_param_info!(RenderGraph, &mut RenderGraph, true);

// Special params
impl_into_param_info!(RenderCommandEncoder, &mut RenderCommandEncoder, false);
impl_into_param_info!(Commands, Commands<'_, '_>, true);
impl_into_param_info!(EventReader, EventReader<'_>, false);
impl_into_param_info!(EventWriter, EventWriter<'_>, true);

// Resources
impl_into_param_info!(R: Resource, Res<R>, false);
impl_into_param_info!(R: Resource, ResMut<R>, true);
impl_into_param_info!(R: Resource, Option<Res<R>>, false);
impl_into_param_info!(R: Resource, Option<ResMut<R>>, true);

// Query components
impl_into_param_info!(EntityId, EntityId, false);
impl_into_param_info!(C: Component, &C, false);
impl_into_param_info!(C: Component, &mut C, true);
impl_into_param_info!(C: Component, Ref<'_, C>, false);
impl_into_param_info!(C: Component, Mut<'_, C>, true);
impl_into_param_info!(C: Component, Option<&C>, false);
impl_into_param_info!(C: Component, Option<&mut C>, true);
impl_into_param_info!(C: Component, Option<Ref<'_, C>>, false);
impl_into_param_info!(C: Component, Option<Mut<'_, C>>, true);

/// Any type that can be used as a system parameter (including tuples of parameters).
/// Implemented for types which can be extracted from the world during system execution.
pub trait SystemParam: IntoParamInfo {
    /// The state stored between system runs.
    type State: Send + Sync + 'static;

    /// Extract the parameter from the world.
    fn extract(world: &mut World, state: &mut Self::State, context: &SystemContext) -> Self;

    /// Apply any changes to the world after system execution on the main thread.
    #[inline]
    fn apply(_world: &mut World, _state: &mut Self::State, _context: &SystemContext) {}

    /// Initialize the parameter state.
    fn init_state() -> Self::State;

    /// Initialize the parameter state with access to the world.
    #[inline]
    fn init_state_world(_world: &mut World) -> Self::State {
        Self::init_state()
    }
}

/// Macro to implement [`SystemParam`] for stateless system parameters with only extract logic
macro_rules! impl_stateless_system_param {
    ($for:ty, $world:ident, $extract:expr) => {
        impl SystemParam for $for {
            type State = ();

            #[inline]
            fn extract(
                $world: &mut World,
                _state: &mut Self::State,
                _context: &SystemContext,
            ) -> Self {
                $extract
            }

            #[inline]
            fn init_state() -> Self::State {}
        }
    };

    ($type:ident: $trait:ident, $for:ty, $world:ident, $context:ident, $extract:expr) => {
        impl<$type: $trait> SystemParam for $for {
            type State = ();

            #[inline]
            fn extract(
                $world: &mut World,
                _state: &mut Self::State,
                $context: &SystemContext,
            ) -> Self {
                $extract
            }

            #[inline]
            fn init_state() -> Self::State {}
        }
    };
}

// Special app params
impl_stateless_system_param!(&mut App, world, unsafe { world.reborrow().parent_app() });
impl_stateless_system_param!(&mut World, world, unsafe { world.reborrow() });
impl_stateless_system_param!(&mut RenderGraph, world, unsafe {
    world.reborrow().parent_app().render_graph()
});

// Special params
impl_stateless_system_param!(EventReader<'_>, world, unsafe {
    world.reborrow().parent_app().events.handlers().0
});
impl_stateless_system_param!(EventWriter<'_>, world, unsafe {
    world.reborrow().parent_app().events.handlers().1
});

// Resources
impl_stateless_system_param!(R: Resource, Res<R>, world, context, {
    let mut res = world.resources.get::<R>();
    res.0.set_last_run(*context.last_run);
    res
});
impl_stateless_system_param!(R: Resource, ResMut<R>, world, context, {
    let mut res_mut = world.resources.get_mut::<R>();
    res_mut.0.set_last_run(*context.last_run);
    res_mut
});
impl_stateless_system_param!(R: Resource, Option<Res<R>>, world, context, {
    let mut option_res = world.resources.try_get::<R>();
    if let Some(res) = option_res.as_mut() {
        res.0.set_last_run(*context.last_run);
    }
    option_res
});
impl_stateless_system_param!(R: Resource, Option<ResMut<R>>, world, context, {
    let mut option_res_mut = world.resources.try_get_mut::<R>();
    if let Some(res_mut) = option_res_mut.as_mut() {
        res_mut.0.set_last_run(*context.last_run);
    }
    option_res_mut
});

impl SystemParam for &mut RenderCommandEncoder {
    type State = Option<RenderCommandEncoder>;
    #[inline]
    fn extract(world: &mut World, state: &mut Self::State, _context: &SystemContext) -> Self {
        debug_assert!(
            state.is_none(),
            "RenderCommandEncoder parameter state should be None on extract"
        );

        if state.is_none() {
            let device = world.resources.get::<RenderDevice>();
            // TODO: make label based on system name
            *state = Some(RenderCommandEncoder::new(&device, "Render Encoder"));
        }

        // Reborrow to satisfy lifetime requirements
        let state = unsafe { &mut *(state as *mut Self::State) };
        unsafe { state.as_mut().unwrap_unchecked() }
    }

    #[inline]
    fn apply(world: &mut World, state: &mut Self::State, _context: &SystemContext) {
        if let Some(encoder) = state.take() {
            world.render_command_queue.push(encoder);
        }
    }

    #[inline]
    fn init_state() -> Self::State {
        None
    }
}

/// State for [`Commands`] system parameter, implemented as a wrapper with Send + Sync
#[derive(Default)]
pub struct CommandsState(CommandQueue);
unsafe impl Send for CommandsState {}
unsafe impl Sync for CommandsState {}

impl SystemParam for Commands<'_, '_> {
    type State = CommandsState;
    #[inline]
    fn extract(world: &mut World, state: &mut Self::State, _context: &SystemContext) -> Self {
        // Reborrow to satisfy lifetime requirements
        let world = unsafe { world.reborrow() };
        let state = unsafe { &mut *(state as *mut Self::State) };

        Commands::new(&mut world.entities.tracking, &mut state.0)
    }

    #[inline]
    fn apply(world: &mut World, state: &mut Self::State, _context: &SystemContext) {
        world.command_queue.extend(&mut state.0);
    }

    #[inline]
    fn init_state() -> Self::State {
        CommandsState::default()
    }
}

pub struct QueryCache; // Placeholder for query state

impl<T, F> SystemParam for Query<T, F>
where
    F: QueryFilter,
    Query<T, F>: IntoParamInfo,
{
    type State = QueryCache; // Placeholder for query state

    #[inline]
    fn extract(world: &mut World, _state: &mut Self::State, context: &SystemContext) -> Self {
        Query::new(&mut world.entities, *context.last_run)
    }

    #[inline]
    fn init_state() -> Self::State {
        QueryCache
    }

    #[inline]
    fn init_state_world(_world: &mut World) -> Self::State {
        QueryCache
    }
}

impl<T, F> IntoParamInfo for Query<T, F>
where
    T: IntoParamInfo,
    F: QueryFilter,
{
    fn params_info() -> Vec<ParamInfo> {
        T::params_info()
    }
}

/// Macros for implementing SystemParam for tuples of different sizes
pub(super) mod macros {
    /// Implement `SystemParam` recursively for tuples of system parameters
    macro_rules! impl_system_param_tuple {
        ($(($($param:ident),*)),*) => {
            $(
                impl<$($param),*> SystemParam for ($($param,)*)
                where
                    $($param: SystemParam,)*
                {
                    type State = ($($param::State,)*);

                    #[allow(unused_variables)]
                    #[inline]
                    fn extract(world: &mut World, state: &mut Self::State, context: &SystemContext) -> Self {
                        #[allow(non_snake_case)]
                        let ($($param,)*) = state;

                        #[allow(clippy::unused_unit)]
                        (
                            $(
                                $param::extract(unsafe { world.reborrow() }, $param, context ),
                            )*
                        )
                    }

                    #[inline]
                    fn init_state() -> Self::State {
                        #[allow(clippy::unused_unit)]
                        ($($param::init_state(),)*)
                    }

                    #[allow(unused_variables)]
                    #[inline]
                    fn init_state_world(world: &mut World) -> Self::State {
                        #[allow(clippy::unused_unit)]
                        ($($param::init_state_world(unsafe { world.reborrow() }),)*)
                    }
                }

                impl<$($param),*> IntoParamInfo for ($($param,)*)
                where
                    $($param: IntoParamInfo,)*
                {
                    fn params_info() -> Vec<ParamInfo> {
                        #[allow(unused_mut)]
                        let mut ids = Vec::new();
                        $(
                            ids.extend($param::params_info());
                        )*
                        ids
                    }
                }
            )*
        }
    }

    pub(crate) use impl_system_param_tuple;
}
