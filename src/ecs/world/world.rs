use crate::query::Query;

use super::entities::Entities;
use super::super::resources::Resources;

pub struct World {
    pub entities: Entities,
    pub resources: Resources,
}

impl World {
    pub fn new() -> Self {
        let mut resources = Resources::new();
        resources.insert_default_resources();

        Self {
            entities: Entities::new(),
            resources,
        }
    }

    /// Creates new world query
    pub fn query<T>(&mut self) -> Query<T> {
        Query::new(&mut self.entities)
    }

    /// Creates new world query with filters
    pub fn query_filtered<T, F>(&mut self) -> Query<T, F> {
        Query::new(&mut self.entities)
    }
}
