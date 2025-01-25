use super::EntityId;

/// A component which holds all the [parents](Parent) children. It's automatically inserted (and removed) if
/// an [entity](super) has at least 1 child.
pub struct Children {
    pub ids: Vec<EntityId>,
}

/// A component added on a [child](Children) entity to store the relation with its parent.
pub struct Parent {
    pub id: EntityId,
}

impl Parent {
    pub fn new(id: EntityId) -> Self {
        Self { id }
    }

    pub fn set(&mut self, id: EntityId) {
        self.id = id;
    }
}

impl Children {
    pub fn new(ids: Vec<EntityId>) -> Self {
        Self { ids }
    }

    pub fn add(&mut self, id: EntityId) {
        if self.ids.contains(&id) {
            return;
        }

        self.ids.push(id);
    }

    pub fn remove(&mut self, id: EntityId) {
        self.ids.retain(|&x| x != id);
    }
}
