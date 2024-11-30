mod run;

pub use run::RunQuery;

use crate::{world::entities::Entities};

pub struct Query<'a, T> {
    entities: &'a mut Entities,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Query<'_, T> {
    pub fn new(entities: &mut Entities) -> Query<T> {
        Query {
            entities,
            _marker: std::marker::PhantomData,
        }
    }
}
