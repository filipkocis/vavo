use std::{
    any::{Any, TypeId}, collections::HashMap, hash::Hash, ops::{Add, Sub}
};

use crate::query::filter::Filters;

use super::{archetype::{Archetype, ArchetypeId}, Children, Parent};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(u32);

impl EntityId {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub(crate) fn raw(&self) -> u32 {
        self.0
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
pub struct Entities {
    next_entity_id: EntityId,
    archetypes: HashMap<ArchetypeId, Archetype>, // Map archetype ID to its storage
    current_tick: *const u64,
}


// TODO: Implement the correct removal and transfer of ticks, not just components

impl Entities {
    /// Create new `Entities` manager with uninitialized tick pointer
    pub fn new() -> Self {
        Self {
            next_entity_id: EntityId(0),
            archetypes: HashMap::new(),
            current_tick: std::ptr::null(),
        }
    }

    /// Initialize tick pointer, necessary for entity creation. Done in `TimePlugin`
    pub fn initialize_tick(&mut self, current_tick: *const u64) {
        self.current_tick = current_tick
    }

    /// Step next entity ID counter
    /// Returns new entity ID
    fn step_entity_id(&mut self) -> EntityId {
        self.next_entity_id = self.next_entity_id + 1;
        self.next_entity_id
    }

    /// Returns archetypes containing type_ids  
    pub(crate) fn archetypes_filtered(&mut self, type_ids: &[TypeId], filters: &Filters) -> Vec<&mut Archetype> {
        self.archetypes.values_mut().filter(|a| {
            a.has_types(type_ids) && a.matches_filters(filters)
        }).collect()
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
            .or_insert_with(|| Archetype::new(types, self.current_tick))
            .insert_entity(entity_id, components);
    }

    /// Despawn entity
    pub(crate) fn despawn_entity(&mut self, entity_id: EntityId) {
        let mut emptied_archetype = None;
        if let Some((id, archetype)) = self.archetypes.iter_mut().find(|(_, a)| a.entity_ids.contains(&entity_id)) {
            archetype.remove_entity(entity_id);

            if archetype.len() == 0 {
                emptied_archetype = Some(*id);
            }
        }

        if let Some(id) = emptied_archetype {
            self.archetypes.remove(&id);
        }
    }

    /// Despawn entity and all its children recursively
    pub(crate) fn despawn_entity_recursive(&mut self, entity_id: EntityId) {
        if let Some(children) = self.get_component::<Children>(entity_id) {
            for child_id in children.ids.clone() {
                self.despawn_entity_recursive(child_id);
            }
        }

        self.despawn_entity(entity_id);
    }

    /// Insert new component
    ///
    /// # Panics
    /// Panics if components type_id is EntityId
    pub(crate) fn insert_component(&mut self, entity_id: EntityId, component: Box<dyn Any>) {
        let type_id = (*component).type_id(); 
        assert_ne!(type_id, TypeId::of::<EntityId>(), "Cannot insert EntityId as a component");

        let mut emptied_archetype = None;
        if let Some((id, archetype)) = self.archetypes.iter_mut().find(|(_, a)| a.entity_ids.contains(&entity_id)) {
            if archetype.has_type(&type_id) {
                assert!(archetype.update_component(entity_id, component), "Failed to update component");
                return;
            }

            let mut old_components = archetype.remove_entity(entity_id).expect("entity_id should exist in archetype");
            old_components.push(component);

            if archetype.len() == 0 {
                emptied_archetype = Some(*id);
            }

            let mut types = archetype.types_vec();
            types.push(type_id);

            let archetype_id = Archetype::hash_types(types.clone());
            self.archetypes.entry(archetype_id)
                .or_insert_with(|| Archetype::new(types, self.current_tick))
                .insert_entity(entity_id, old_components);
        }

        if let Some(id) = emptied_archetype {
            self.archetypes.remove(&id);
        }
    }

    /// Remove component
    ///
    /// # Panics
    /// Panics if type_id is EntityId
    pub(crate) fn remove_component(&mut self, entity_id: EntityId, type_id: TypeId) {
        assert_ne!(type_id, TypeId::of::<EntityId>(), "Cannot remove builtin EntityId component");

        let mut emptied_archetype = None;
        if let Some((id, archetype)) = self.archetypes.iter_mut().find(|(_, a)| a.entity_ids.contains(&entity_id)) {
            if !archetype.has_type(&type_id) {
                return;
            }

            let mut old_components = archetype.remove_entity(entity_id).expect("entity_id should exist in archetype");
            let index = archetype.types[&type_id];
            old_components.remove(index);

            if archetype.len() == 0 {
                emptied_archetype = Some(*id);
            }

            let mut types = archetype.types_vec();
            types.remove(index);

            let archetype_id = Archetype::hash_types(types.clone());
            self.archetypes.entry(archetype_id)
                .or_insert_with(|| Archetype::new(types, self.current_tick))
                .insert_entity(entity_id, old_components);
        }

        if let Some(id) = emptied_archetype {
            self.archetypes.remove(&id);
        }
    }

    /// Get component mutably
    pub(crate) fn get_component_mut<T: 'static>(&mut self, entity_id: EntityId) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        for archetype in self.archetypes.values_mut() {
            let entity_index = match archetype.get_entity_index(entity_id) {
                Some(index) => index,
                None => continue,
            };

            if let Some(component_index) = archetype.types.get(&type_id).cloned() {
                archetype.mark_mutated_single(entity_index, component_index);
                let component = &mut archetype.components[component_index][entity_index];
                return component.downcast_mut::<T>();
            } else {
                return None
            }
        }

        None
    }

    /// Get component
    pub(crate) fn get_component<T: 'static>(&self, entity_id: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        for archetype in self.archetypes.values() {
            let entity_index = match archetype.get_entity_index(entity_id) {
                Some(index) => index,
                None => continue,
            };

            if let Some(component_index) = archetype.types.get(&type_id) {
                let component = &archetype.components[*component_index][entity_index];
                return component.downcast_ref::<T>();
            } else {
                return None
            }
        }

        None
    }

    /// Add child to parent's Children component, and add Parent component to child
    ///
    /// # Panics
    /// Panics if parent or child does not exist, or if child == parent
    pub(crate) fn add_child(&mut self, parent_id: EntityId, child_id: EntityId) {
        assert_ne!(parent_id, child_id, "Parent and child cannot be the same entity");
        assert!(self.archetypes.values().any(|a| a.entity_ids.contains(&parent_id)), "Parent entity does not exist");
        assert!(self.archetypes.values().any(|a| a.entity_ids.contains(&child_id)), "Child entity does not exist");

        if let Some(children) = self.get_component_mut::<Children>(parent_id) {
            children.add(child_id);
        } else {
            self.insert_component(parent_id, Box::new(Children::new(vec![child_id])));
        }

        self.insert_component(child_id, Box::new(Parent::new(parent_id)));
    }

    /// Remove child from parent's Children component, and remove Parent component from child
    pub(crate) fn remove_child(&mut self, parent_id: EntityId, child_id: EntityId) {
        if let Some(children) = self.get_component_mut::<Children>(parent_id) {
            if children.ids.contains(&child_id) {
                children.remove(child_id);
                self.remove_component(child_id, TypeId::of::<Parent>());
            }
        }
    }
}
