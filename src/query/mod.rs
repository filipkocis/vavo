mod run;
pub mod filter;

pub use run::RunQuery;

use crate::{world::entities::Entities};

pub struct Query<'a, T, F = ()> {
    entities: &'a mut Entities,
    _marker: std::marker::PhantomData<(T, F)>,
}

impl<T, F> Query<'_, T, F> {
    pub(crate) fn new(entities: &mut Entities) -> Query<T, F> {
        Query {
            entities,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn cast<'a, U, V>(&'a mut self) -> Query<'a, U, V> {
        Query {
            entities: self.entities,
            _marker: std::marker::PhantomData
        }
    }
}
