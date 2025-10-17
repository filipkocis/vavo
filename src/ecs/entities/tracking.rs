#[cfg(debug_assertions)]
use std::collections::HashSet;

use crate::{ecs::entities::ArchetypeId, prelude::EntityId};

/// Tracked location of an [entity](super::EntityId) in the [Entities](super::Entities) storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Debug set of freed ids for easier tracking of errors
    #[cfg(debug_assertions)]
    debug_free_ids: HashSet<EntityId>,
    /// Debug set of assigned locations for easier tracking of errors
    #[cfg(debug_assertions)]
    debug_locations: HashSet<EntityLocation>,
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
            #[cfg(debug_assertions)]
            self.debug_free_ids.remove(&id);
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

        debug_assert!(
            index < self.locations.len(),
            "Trying to set location for untracked entity id {:?}",
            entity
        );

        debug_assert!(
            !self.debug_locations.contains(&location),
            "Entity location {:?} is already assigned to another entity",
            location
        );
        #[cfg(debug_assertions)]
        {
            self.debug_locations.insert(location);
            if let Some(previous) = &self.locations[index] {
                self.debug_locations.remove(previous);
            }
        }

        debug_assert!(
            !self.debug_free_ids.contains(&entity),
            "Trying to set location for freed entity id {:?}",
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
            #[cfg(debug_assertions)]
            {
                self.debug_free_ids.insert(entity);
                if let Some(loc) = location {
                    self.debug_locations.remove(&loc);
                }
            }
            location
        } else {
            None
        }
    }
}
