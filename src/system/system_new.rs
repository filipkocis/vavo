use crate::{
    prelude::{Component, Mut, Ref, Res, ResMut, Resource, Tick, World},
    query::{Query, RunQuery, filter::QueryFilter},
    system::Commands,
};
use std::{
    any::{TypeId, type_name},
    collections::HashMap,
};

// --------------------------- //
//      System Definition      //
// --------------------------- //

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

pub type SystemExecFn = dyn FnMut(&mut World) + Send + Sync + 'static;
pub struct SystemExec {
    /// Function's parameters info
    pub params_info: Vec<ParamInfo>,
    /// Function's type info
    pub exec_info: TypeInfo,
    /// System execution function
    exec: Box<SystemExecFn>,
}

impl SystemExec {
    /// Create a new system execution from function `exec` and its type information
    pub fn new(params_info: Vec<ParamInfo>, exec_info: TypeInfo, exec: Box<SystemExecFn>) -> Self {
        Self {
            params_info,
            exec_info,
            exec,
        }
    }

    /// Execute the system function
    #[inline]
    pub fn run(&mut self, world: &mut World) {
        (self.exec)(world);
    }
}

pub struct System {
    /// Tick of the last run, or `0`
    last_run: Tick,
    /// System execution
    exec: SystemExec,
    /// Run conditions
    conditions: Vec<SystemCondition>,
}

impl System {
    /// Same as [`IntoSystem::run_if`] but internal to avoid the need for generic parameters
    #[inline]
    fn internal_run_if(mut self, condition: SystemCondition) -> System {
        self.conditions.push(condition);
        self
    }

    /// Returns tick of the last run
    #[inline]
    pub fn last_run(&self) -> Tick {
        self.last_run
    }

    /// Execute system if all conditions are met
    pub fn run(&mut self, world: &mut World) {
        // TODO: handle world tick overflow
        if self.satisfies_conditions(world) {
            // Increment must come first to ensure `system.last_run < world.tick`
            world.tick.increment();
            self.exec.run(world);
            self.last_run = *world.tick;
        }
    }

    /// Check if all run conditions are satisfied
    fn satisfies_conditions(&mut self, world: &mut World) -> bool {
        self.conditions
            .iter_mut()
            .all(|condition| condition.run(world))
    }
}

pub trait IntoSystem<P: SystemParam> {
    /// Convert self into a [`System`]
    fn build(self) -> System;

    /// Add new run condition to the system
    fn run_if(self, condition: impl IntoSystemCondition) -> System;
}

impl<P: SystemParam> IntoSystem<P> for System {
    #[inline]
    fn build(self) -> System {
        self
    }

    #[inline]
    fn run_if(self, condition: impl IntoSystemCondition) -> System {
        self.internal_run_if(condition.build())
    }
}

// --------------------------- //
//      System Condition       //
// --------------------------- //

type SystemConditionExecFn = dyn FnMut(&mut World) -> bool;
pub struct SystemCondition {
    type_name: &'static str,
    type_id: TypeId,
    last_run: Tick,
    exec: Box<SystemConditionExecFn>,
}

impl SystemCondition {
    /// Returns function's type name
    #[inline]
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Returns function's type id
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Execute the condition function
    #[inline]
    pub fn run(&mut self, world: &mut World) -> bool {
        world.tick.increment();
        let result = (self.exec)(world);
        self.last_run = *world.tick;
        result
    }
}

pub trait IntoSystemCondition {
    /// Convert the function into a [`SystemCondition`]
    fn build(self) -> SystemCondition;
}

impl IntoSystemCondition for SystemCondition {
    #[inline]
    fn build(self) -> SystemCondition {
        self
    }
}

// --------------------------- //
//      System FN impls        //
// --------------------------- //

pub trait IntoParamInfo {
    /// Returns information about the parameter types and their access patterns.
    fn params_info() -> Vec<ParamInfo>;
}

/// Any type that can be used as a system parameter (including tuples of parameters).
/// Implemented for types which can be extracted from the world during system execution.
pub trait SystemParam: IntoParamInfo {
    /// Extract the parameter from the world.
    fn extract(world: &mut World) -> Self;
}

impl SystemParam for &mut World {
    #[inline]
    fn extract(world: &mut World) -> Self {
        unsafe { &mut *(world as *mut _) }
    }
}
impl IntoParamInfo for &mut World {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = true;
        let type_info = TypeInfo::new(type_name::<World>(), TypeId::of::<World>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}

impl SystemParam for Commands<'_, '_> {
    #[inline]
    fn extract(world: &mut World) -> Self {
        // Reborrow world to satisfy lifetime requirements
        let reborrowed = unsafe { &mut *(world as *mut World) };
        reborrowed.commands()
    }
}
impl IntoParamInfo for Commands<'_, '_> {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = true;
        let type_info = TypeInfo::new(type_name::<Commands>(), TypeId::of::<Commands>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}

impl<R: Resource> SystemParam for Res<R> {
    #[inline]
    fn extract(world: &mut World) -> Self {
        world.resources.get()
    }
}
impl<R: Resource> IntoParamInfo for Res<R> {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = false;
        let type_info = TypeInfo::new(type_name::<R>(), TypeId::of::<R>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}

impl<R: Resource> SystemParam for ResMut<R> {
    #[inline]
    fn extract(world: &mut World) -> Self {
        world.resources.get_mut()
    }
}
impl<R: Resource> IntoParamInfo for ResMut<R> {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = true;
        let type_info = TypeInfo::new(type_name::<R>(), TypeId::of::<R>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}

impl<T, F> SystemParam for Query<T, F>
where
    F: QueryFilter,
    Query<T, F>: IntoParamInfo,
{
    #[inline]
    fn extract(world: &mut World) -> Self {
        world.query_filtered::<T, F>()
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
impl IntoParamInfo for EntityId {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = false;
        let type_info = TypeInfo::new(type_name::<EntityId>(), TypeId::of::<EntityId>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}
impl<C: Component> IntoParamInfo for &mut C {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = true;
        let type_info = TypeInfo::new(type_name::<C>(), TypeId::of::<C>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}
impl<C: Component> IntoParamInfo for &C {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = false;
        let type_info = TypeInfo::new(type_name::<C>(), TypeId::of::<C>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}
impl<C: Component> IntoParamInfo for Mut<'_, C> {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = true;
        let type_info = TypeInfo::new(type_name::<C>(), TypeId::of::<C>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}
impl<C: Component> IntoParamInfo for Ref<'_, C> {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = false;
        let type_info = TypeInfo::new(type_name::<C>(), TypeId::of::<C>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}
impl<C: Component> IntoParamInfo for Option<&mut C> {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = true;
        let type_info = TypeInfo::new(type_name::<C>(), TypeId::of::<C>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}
impl<C: Component> IntoParamInfo for Option<&C> {
    fn params_info() -> Vec<ParamInfo> {
        let is_mutable = false;
        let type_info = TypeInfo::new(type_name::<C>(), TypeId::of::<C>());
        vec![ParamInfo::new(is_mutable, type_info)]
    }
}

macro_rules! impl_system_param_tuple {
    ($(($($param:ident),*)),*) => {
        $(
            impl<$($param),*> SystemParam for ($($param,)*)
            where
                $($param: SystemParam,)*
            {
                #[allow(unused_variables)]
                #[inline]
                fn extract(world: &mut World) -> Self {
                    #[allow(clippy::unused_unit)]
                    (
                        $(
                            $param::extract(unsafe { &mut *(world as *mut _) }),
                        )*
                    )
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

/// Check for borrow conflicts among system parameters, returning the first conflicting type.
fn check_borrow_conflicts(params_info: &[ParamInfo]) -> Option<TypeInfo> {
    let mut seen = HashMap::<TypeId, bool>::new();

    for param_info in params_info {
        let is_mutable = param_info.is_mutable();
        let type_info = param_info.type_info();

        if let Some(existing_access) = seen.get(&type_info.type_id()) {
            if param_info.is_mutable() || *existing_access {
                return Some(type_info);
            }
        } else {
            seen.insert(type_info.type_id(), is_mutable);
        }
    }

    None
}

// impl<P1, P2, P3, F> IntoSystem<(P1, P2, P3)> for F
// where
//     P1: SystemParam,
//     P2: SystemParam,
//     P3: SystemParam,
//     F: FnMut(P1, P2, P3) + Send + Sync + 'static,
// {
//     fn build(mut self) -> System {
//         let params_info = <(P1, P2, P3)>::params_info();
//         let exec_info = TypeInfo::new(type_name::<F>(), TypeId::of::<F>());
//
//         if let Some(conflict) = check_borrow_conflicts(&params_info) {
//             panic!(
//                 "System function '{}' has conflicting parameter accesses: {:?}",
//                 exec_info.type_name(),
//                 conflict.type_name(),
//             );
//         }
//
//         let exec_fn = Box::new(move |world: &mut World| {
//             let p1 = P1::extract(unsafe { &mut *(world as *mut _) });
//             let p2 = P2::extract(unsafe { &mut *(world as *mut _) });
//             let p3 = P3::extract(unsafe { &mut *(world as *mut _) });
//             self(p1, p2, p3);
//         });
//
//         let exec = SystemExec::new(params_info, exec_info, exec_fn);
//
//         System {
//             last_run: Tick::default(),
//             exec,
//             conditions: Vec::new(),
//         }
//     }
//
//     #[inline]
//     fn run_if(self, condition: impl IntoSystemCondition) -> System {
//         self.build().internal_run_if(condition.build())
//     }
// }

macro_rules! impl_into_system {
    ($(($($param:ident),*)),*) => {
        $(
            impl<$($param,)* F> IntoSystem<($($param,)*)> for F
            where
                $($param: SystemParam,)*
                F: FnMut($($param),*) + Send + Sync + 'static,
            {
                #[inline]
                fn build(mut self) -> System {
                    let params_info = <( $($param,)* )>::params_info();
                    let exec_info = TypeInfo::new(type_name::<F>(), TypeId::of::<F>());

                    if let Some(conflict) = check_borrow_conflicts(&params_info) {
                        panic!(
                            "System function '{}' has conflicting parameter accesses: {:?}",
                            exec_info.type_name(),
                            conflict.type_name(),
                        );
                    }

                    #[allow(unused_variables)]
                    let exec_fn = Box::new(move |world: &mut World| {
                        $(
                            #[allow(non_snake_case)]
                            let $param = $param::extract(unsafe { &mut *(world as *mut _) });
                        )*
                        self($($param),*);
                    });

                    let exec = SystemExec::new(params_info, exec_info, exec_fn);

                    System {
                        last_run: Tick::default(),
                        exec,
                        conditions: Vec::new(),
                    }
                }

                #[inline]
                fn run_if(self, condition: impl IntoSystemCondition) -> System {
                    self.build().internal_run_if(condition.build())
                }
            }
        )*
    }
}

#[rustfmt::skip]
impl_into_system!(
    (),
    (P1),
    (P1, P2),
    (P1, P2, P3),
    (P1, P2, P3, P4),
    (P1, P2, P3, P4, P5),
    (P1, P2, P3, P4, P5, P6),
    (P1, P2, P3, P4, P5, P6, P7),
    (P1, P2, P3, P4, P5, P6, P7, P8),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16)
);

#[rustfmt::skip]
impl_system_param_tuple!(
    (),
    (P1),
    (P1, P2),
    (P1, P2, P3),
    (P1, P2, P3, P4),
    (P1, P2, P3, P4, P5),
    (P1, P2, P3, P4, P5, P6),
    (P1, P2, P3, P4, P5, P6, P7),
    (P1, P2, P3, P4, P5, P6, P7, P8),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15),
    (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16)
);
