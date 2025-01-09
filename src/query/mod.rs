mod run;
pub mod filter;

pub use run::RunQuery;

use crate::{world::entities::Entities};

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
