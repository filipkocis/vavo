use super::EntityId;

pub struct Children {
    pub ids: Vec<EntityId>,
}

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
