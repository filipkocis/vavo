use std::any::{type_name, TypeId};

use crate::query::Query;

use super::{System, SystemsContext};

pub trait IntoSystem<T, F> {
    /// Convert self into a [`System`]
    fn system(self) -> System;
}

impl IntoSystem<(), ()> for System {
    fn system(self) -> System {
        self
    }
}

impl<E, T, F> IntoSystem<T, F> for E
where
    E: Fn(&mut SystemsContext, Query<T, F>) + 'static,
{
    fn system(self) -> System {
        System::new(self, type_name::<E>(), TypeId::of::<E>())
    }
}

pub trait IntoSystemCondition<T, F> {
    /// Convert the function into a [`SystemCondition`] function
    fn system_condition(self) -> Box<dyn FnMut(&mut SystemsContext, Query<T, F>) -> bool>;
}

impl<E, T, F> IntoSystemCondition<T, F> for E
where
    E: FnMut(&mut SystemsContext, Query<T, F>) -> bool + 'static,
{
    fn system_condition(self) -> Box<dyn FnMut(&mut SystemsContext, Query<T, F>) -> bool> {
        Box::new(self)
    }
}
