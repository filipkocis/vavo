use crate::{ecs::entities::ArchetypeId, prelude::EntityId};

/// Tracked location of an [entity](super::EntityId) in the [Entities](super::Entities) storage.
#[derive(Debug, Clone, Copy)]
pub struct EntityLocation {
    /// Archetype containing the entity
    archetype_id: ArchetypeId,
    /// Index within the archetype
    index: usize,
}

impl EntityLocation {
    /// Create new location
    #[inline]
    pub fn new(archetype_id: ArchetypeId, index: usize) -> Self {
        Self {
            archetype_id,
            index,
        }
    }

    /// Returns the archetype id
    #[inline]
    pub fn archetype_id(&self) -> ArchetypeId {
        self.archetype_id
    }

    /// Returns the index within the archetype
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }
}

/// Main storage for generating new [entity](super::EntityId) ids and tracking their locations. It
/// stores a list of free ids to be reused and their precise location in archetypes, if any.
#[derive(Debug, Default)]
pub struct EntityTracking {
    /// List of ids freed for reuse
    free_ids: Vec<EntityId>,
    /// Locations of tracked entities
    locations: Vec<Option<EntityLocation>>,
}

impl EntityTracking {
    /// Create new empty tracking storage
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a new entity id, reusing a freed one with an incremented generation if possible.
    #[inline]
    pub fn new_id(&mut self) -> EntityId {
        if let Some(id) = self.free_ids.pop() {
            EntityId::new(id.index(), id.generation() + 1)
        } else {
            let next_id = self.locations.len() as u32;
            self.locations.push(None);
            EntityId::new(next_id, 0)
        }
    }

    /// Sets the location for an entity
    ///
    /// # Panics
    /// Panics if the entity id is not tracked
    #[inline]
    pub fn set_location(&mut self, entity: EntityId, location: EntityLocation) {
        let index = entity.index() as usize;

        assert!(
            index < self.locations.len(),
            "Trying to set location for untracked entity id {:?}",
            entity
        );

        self.locations[index] = Some(location);
    }

    /// Gets the location for an entity
    #[inline]
    pub fn get_location(&self, entity: EntityId) -> Option<EntityLocation> {
        let index = entity.index() as usize;
        if index < self.locations.len() {
            self.locations[index]
        } else {
            None
        }
    }

    /// Removes an entity from tracking, freeing its id for reuse.
    /// Returns the previous location of the entity, if any.
    #[inline]
    pub fn remove_entity(&mut self, entity: EntityId) -> Option<EntityLocation> {
        let index = entity.index() as usize;
        if index < self.locations.len() {
            let location = self.locations[index].take();
            self.free_ids.push(entity);
            location
        } else {
            None
        }
    }
}
