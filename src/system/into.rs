use std::{any::TypeId, collections::HashMap};

use super::{ParamInfo, System, SystemCondition, SystemParam, TypeInfo};

/// Convert a closure or function into a [`System`]
pub trait IntoSystem<P: SystemParam> {
    /// Convert self into a [`System`]
    fn build(self) -> System;

    /// Add new run condition to the system
    fn run_if<CP: SystemParam>(self, condition: impl IntoSystemCondition<CP>)
    -> impl IntoSystem<P>;
}

impl<P: SystemParam> IntoSystem<P> for System {
    #[inline]
    fn build(self) -> System {
        self
    }

    #[inline]
    fn run_if<CP: SystemParam>(
        self,
        condition: impl IntoSystemCondition<CP>,
    ) -> impl IntoSystem<P> {
        self.internal_run_if(condition.build())
    }
}

/// Convert a closure or function into a [`SystemCondition`]
pub trait IntoSystemCondition<P: SystemParam> {
    /// Convert the function into a [`SystemCondition`]
    fn build(self) -> SystemCondition;
}

impl<P: SystemParam> IntoSystemCondition<P> for SystemCondition {
    #[inline]
    fn build(self) -> SystemCondition {
        self
    }
}

/// Check for borrow conflicts among system parameters, returning the first conflicting type.
pub fn check_borrow_conflicts(params_info: &[ParamInfo]) -> Option<TypeInfo> {
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

/// Macros for implementing `IntoSystem` and `IntoSystemCondition` for different parameter counts
pub(super) mod macros {
    pub use super::super::params::*;
    pub use super::super::*;
    pub use super::check_borrow_conflicts;
    pub use crate::prelude::*;
    pub use std::any::{TypeId, type_name};

    /// Convert a closure or function into a [`SystemCondition`]
    macro_rules! impl_into_system_condition {
        ($(($($param:ident),*)),*) => {
            $(
                impl<$($param,)* F> IntoSystemCondition<($($param,)*)> for F
                where
                    $($param: SystemParam,)*
                    F: FnMut($($param),*) -> bool + Send + Sync + 'static,
                {
                    #[inline]
                    fn build(mut self) -> SystemCondition {
                        #![allow(non_snake_case)]

                        let exec = into_systems_impl_body!(self, ($($param),*) );

                        SystemCondition {
                            last_run: Tick::default(),
                            exec,
                        }
                    }
                }
            )*
        }
    }

    /// Convert a closure or function into a [`System`]
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
                        #![allow(non_snake_case)]

                        let exec = into_systems_impl_body!(self, ($($param),*) );

                        System {
                            last_run: Tick::default(),
                            exec,
                            conditions: Vec::new(),
                        }
                    }

                    #[inline]
                    fn run_if<CP: SystemParam>(
                        self,
                        condition: impl IntoSystemCondition<CP>
                    ) -> impl IntoSystem<($($param,)*)> {
                        self.build().internal_run_if(condition.build())
                    }
                }
            )*
        }
    }

    /// Common body for `IntoSystem` and `IntoSystemCondition` macro implementations
    macro_rules! into_systems_impl_body {
        ($self:ident, ($($param:ident),*)) => {{
            let params_info = <( $($param,)* )>::params_info();
            let exec_info = TypeInfo::new(type_name::<F>(), TypeId::of::<F>());

            if let Some(conflict) = check_borrow_conflicts(&params_info) {
                panic!(
                    "System function '{}' has conflicting parameter accesses: {:?}",
                    exec_info.type_name(),
                    conflict.type_name(),
                );
            }

            // Initialize parameter states into a tuple
            $( let mut $param = Box::new($param::init_state()); )*

            // Safety: These are used within the 'apply' closure, which is on the main
            // thread after all systems have finished, so 'exec' will not be running
            #[allow(clippy::unused_unit, unused_mut, unused_unsafe)]
            let mut unsafe_states_copy = unsafe { ( $(&mut *(&mut *$param as *mut _),)* ) };

            #[allow(unused_variables)]
            let exec_fn = Box::new(move |world: &mut World, context: SystemContext| {
                $(
                    let $param = $param::extract(unsafe { world.reborrow() }, &mut $param, &context);
                )*
                $self($($param),*)
            });

            #[allow(unused_variables)]
            let apply_fn = Box::new(move |world: &mut World, context: SystemContext| {
                let ( $(ref mut $param,)* ) = unsafe_states_copy;

                $(
                    $param::apply(unsafe { world.reborrow() }, $param, &context);
                )*
            });

            let exec = SystemExec::new(params_info, exec_info, exec_fn, apply_fn);
            exec
        }}
    }

    pub(crate) use impl_into_system;
    pub(crate) use impl_into_system_condition;
    pub(crate) use into_systems_impl_body;
}
