mod run;
pub mod filter;

use std::any::TypeId;

pub use run::RunQuery;

use crate::ecs::entities::Entities;

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
    pub fn is_option(&self) -> bool {
        match self {
            QueryComponentType::Option(_) => true,
            _ => false,
        }
    }

    /// Returns the underlying [`component`](crate::ecs::components::Component) type id.
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
    _marker: std::marker::PhantomData<(T, F)>,
}

impl<T, F> Query<T, F> {
    pub(crate) fn new(entities: &mut Entities) -> Query<T, F> {
        Query {
            entities,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn cast<U, V>(&mut self) -> Query<U, V> {
        Query {
            entities: self.entities,
            _marker: std::marker::PhantomData
        }
    }
}
