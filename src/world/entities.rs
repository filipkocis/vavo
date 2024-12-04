use std::{
    any::{Any, TypeId}, collections::HashMap, hash::Hash, ops::{Add, Sub}
};

use super::archetype::{Archetype, ArchetypeId};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(u32);

impl EntityId {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
}

impl Add<u32> for EntityId {
    type Output = EntityId;

    fn add(self, rhs: u32) -> Self::Output {
        EntityId(self.0 + rhs)
    }
}

impl Sub<u32> for EntityId {
    type Output = EntityId;

    fn sub(self, rhs: u32) -> Self::Output {
        EntityId(self.0 - rhs)
    }
}

#[derive(Debug)]
pub(crate) struct Entities {
    next_entity_id: EntityId,
    archetypes: HashMap<ArchetypeId, Archetype>, // Map archetype ID to its storage
}

impl Entities {
    pub fn new() -> Self {
        Self {
            next_entity_id: EntityId(0),
            archetypes: HashMap::new(),
        }
    }

    /// Step next entity ID counter
    /// Returns new entity ID
    fn step_entity_id(&mut self) -> EntityId {
        self.next_entity_id = self.next_entity_id + 1;
        self.next_entity_id
    }

    /// Returns archetypes containing type_ids  
    pub(crate) fn archetypes_filtered(&mut self, type_ids: &[TypeId]) -> Vec<&mut Archetype> {
        self.archetypes.values_mut().filter(|a| a.has_types(type_ids)).collect()
    }

    /// Exposes next entity ID
    pub(crate) fn next_entity_id(&self) -> EntityId {
        self.next_entity_id
    }

    /// Spawn new entity with components
    ///
    /// # Panic
    /// Panics if components contain EntityId
    pub(crate) fn spawn_entity(&mut self, entity_id: EntityId, mut components: Vec<Box<dyn Any>>) {
        let entity_id_type = TypeId::of::<EntityId>();
        assert!(
            !components.iter().any(|c| (**c).type_id() == entity_id_type),
            "Cannot insert EntityId as a component"
        );

        components.push(Box::new(entity_id));
        let types = components.iter().map(|c| (**c).type_id()).collect::<Vec<_>>();
        let archetype_id = Archetype::hash_types(types.clone()); 

        assert!(
            self.next_entity_id == entity_id, 
            "Entity ID mismatch with next entity ID (id {:?} != next {:?})", 
            entity_id, self.next_entity_id
        );
        self.step_entity_id();

        self.archetypes.entry(archetype_id)
            .or_insert_with(|| Archetype::new(types))
            .insert_entity(entity_id, components);
    }

    /// Despawn entity
    pub(crate) fn despawn_entity(&mut self, entity_id: EntityId) {
        if let Some(archetype) = self.archetypes.values_mut().find(|a| a.entity_ids.contains(&entity_id)) {
            archetype.remove_entity(entity_id);
        }
    }

    /// Insert new component
    ///
    /// # Panics
    /// Panics if components type_id is EntityId
    pub(crate) fn insert_component(&mut self, entity_id: EntityId, component: Box<dyn Any>) {
        let type_id = (*component).type_id(); 
        assert_ne!(type_id, TypeId::of::<EntityId>(), "Cannot insert EntityId as a component");

        if let Some(archetype) = self.archetypes.values_mut().find(|a| a.entity_ids.contains(&entity_id)) {
            if archetype.has_type(type_id) {
                assert!(archetype.update_component(entity_id, component), "Failed to update component");
                return;
            }

            let mut old_components = archetype.remove_entity(entity_id).expect("entity_id should exist in archetype");
            old_components.push(component);

            let mut types = archetype.types_vec();
            types.push(type_id);

            let archetype_id = Archetype::hash_types(types.clone());
            self.archetypes.entry(archetype_id)
                .or_insert_with(|| Archetype::new(types))
                .insert_entity(entity_id, old_components);
        }
    }

    /// Remove component
    ///
    /// # Panics
    /// Panics if type_id is EntityId
    pub(crate) fn remove_component(&mut self, entity_id: EntityId, type_id: TypeId) {
        assert_ne!(type_id, TypeId::of::<EntityId>(), "Cannot remove builtin EntityId component");

        if let Some(archetype) = self.archetypes.values_mut().find(|a| a.entity_ids.contains(&entity_id)) {
            if !archetype.has_type(type_id) {
                return;
            }

            let mut old_components = archetype.remove_entity(entity_id).expect("entity_id should exist in archetype");
            let index = archetype.types[&type_id];
            old_components.remove(index);

            let mut types = archetype.types_vec();
            types.remove(index);

            let archetype_id = Archetype::hash_types(types.clone());
            self.archetypes.entry(archetype_id)
                .or_insert_with(|| Archetype::new(types))
                .insert_entity(entity_id, old_components);
        }
    }
}
