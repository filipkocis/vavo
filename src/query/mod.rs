pub mod filter;
mod run;

use std::any::TypeId;

pub use run::RunQuery;

use crate::{ecs::entities::Entities, prelude::Tick};

/// Holds different types of requested [`component`](crate::ecs::components::Component) types in a query. Used to differentiate between normal
/// references and `Option<Component>`.
pub(crate) enum QueryComponentType {
    /// Can be `&Component`, `&mut Component` or owned `EntityId`.
    Normal(TypeId),
    /// Represents the inner type of either `Option<&Component>` or `Option<&mut Component>`.
    Option(TypeId),
}

impl QueryComponentType {
    /// True if component was requested as an `Option<T>`.
    #[inline]
    pub fn is_option(&self) -> bool {
        matches!(self, QueryComponentType::Option(_))
    }

    /// Returns the underlying [`component`](crate::ecs::components::Component) type id.
    #[inline]
    pub fn get_inner_type(&self) -> &TypeId {
        match self {
            QueryComponentType::Normal(inner) => inner,
            QueryComponentType::Option(inner) => inner,
        }
    }
}

pub struct Query<T, F = ()> {
    /// World's entities raw pointer to bypass lifetime limitations.
    ///
    /// # Note
    /// One problem it creates is that query.iter() makes it possible to create multiple mutable
    /// references to the same entity and add them e.g. to resources, which is not allowed but is
    /// possible with this approach.
    ///
    /// # Safety
    /// Always valid
    entities: *mut Entities,
    /// Each system execution context provides its own `last_run` tick.
    system_last_run: Tick,
    _marker: std::marker::PhantomData<(T, F)>,
}

impl<T, F> Query<T, F> {
    #[inline]
    pub(crate) fn new(entities: &mut Entities, system_last_run: Tick) -> Query<T, F> {
        Query {
            entities,
            system_last_run,
            _marker: std::marker::PhantomData,
        }
    }

    /// Creates a new query with a diffrent set of components and filters.
    ///
    /// It is possible to query for the same component multiple times, even as a mutable reference
    /// so you must be careful with this method.
    #[inline]
    pub fn cast<U, V>(&mut self) -> Query<U, V> {
        Query {
            entities: self.entities,
            system_last_run: self.system_last_run,
            _marker: std::marker::PhantomData,
        }
    }
}
