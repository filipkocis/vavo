pub mod archetype;
pub mod relation;
pub mod components;

pub use components::Component;

use std::{
    any::{Any, TypeId}, collections::HashMap, hash::Hash, ops::{Add, Sub}
};

use crate::query::{filter::Filters, QueryComponentType};
use crate::macros::{Component, Reflect};

use archetype::{Archetype, ArchetypeId};
use relation::{Children, Parent};

use super::tick::Tick;

/// Unique identifier for an [entity](Entities) in a [`World`](crate::ecs::world::World)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub struct EntityId(u32);

impl EntityId {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub(crate) fn raw(self) -> u32 {
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
/// Entity store, manages archetypes and all their entities (components) in the `world`
pub struct Entities {
    next_entity_id: EntityId,
    archetypes: HashMap<ArchetypeId, Archetype>, // Map archetype ID to its storage
    current_tick: *const Tick,
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

    /// Exposes archetyeps
    pub fn archetypes(&self) -> impl Iterator<Item = &Archetype> { 
        self.archetypes.values().into_iter()
    }

    /// Initialize tick pointer, necessary for entity creation. Done in
    /// [`World`](crate::prelude::World) initialization.
    pub fn initialize_tick(&mut self, current_tick: *const Tick) {
        self.current_tick = current_tick
    }

    /// Step next entity ID counter
    /// Returns new entity ID
    fn step_entity_id(&mut self) -> EntityId {
        self.next_entity_id = self.next_entity_id + 1;
        self.next_entity_id
    }

    /// Returns archetypes with matching [`query types`](QueryComponentType) and filters, and component indices for
    /// `changed` filters acquired from [`Archetype::get_changed_filter_indices`]
    pub(crate) fn archetypes_filtered<'a>(&'a mut self, type_ids: &'a [QueryComponentType], filters: &'a mut Filters) -> impl Iterator<Item = (&'a mut Archetype, Vec<Vec<usize>>)> {
        self.archetypes.values_mut().filter_map(|a| {
            if a.has_query_types(type_ids) && a.matches_filters(filters) {
                let indices = a.get_changed_filter_indices(filters);
                Some((a, indices))
            } else {
                None
            }
        })
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

    /// Despawn entity and break all relations
    pub(crate) fn despawn_entity(&mut self, entity_id: EntityId) {
        // Remove relations
        if let Some(parent) = self.get_component::<Parent>(entity_id) {
            self.remove_child(parent.id, entity_id);
        }

        // Remove entity
        let mut emptied_archetype = None;
        if let Some((id, archetype)) = self.archetypes.iter_mut().find(|(_, a)| a.has_entity(&entity_id)) {
            archetype.remove_entity(entity_id);

            if archetype.len() == 0 {
                emptied_archetype = Some(*id);
            }
        }

        // Remove archetype if empty
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

    /// Insert new component, or replace existing one
    ///
    /// # Panics
    /// Panics if components type_id is EntityId
    pub(crate) fn insert_component(&mut self, entity_id: EntityId, component: Box<dyn Any>, replace: bool) {
        let type_id = (*component).type_id(); 
        assert_ne!(type_id, TypeId::of::<EntityId>(), "Cannot insert EntityId as a component");

        let mut emptied_archetype = None;
        if let Some((id, archetype)) = self.archetypes.iter_mut().find(|(_, a)| a.has_entity(&entity_id)) {
            if archetype.has_type(&type_id) {
                if replace {
                    assert!(archetype.update_component(entity_id, component), "Failed to update component");
                }
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
        if let Some((id, archetype)) = self.archetypes.iter_mut().find(|(_, a)| a.has_entity(&entity_id)) {
            let Some(component_index) = archetype.get_component_index(&type_id) else {
                return;
            };

            let mut old_components = archetype.remove_entity(entity_id).expect("entity_id should exist in archetype");
            old_components.remove(component_index);

            if archetype.len() == 0 {
                emptied_archetype = Some(*id);
            }

            let mut types = archetype.types_vec();
            types.remove(component_index);

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
    pub(crate) fn get_component_mut<C: Component>(&mut self, entity_id: EntityId) -> Option<&mut C> {
        let type_id = TypeId::of::<C>();
        for archetype in self.archetypes.values_mut() {
            let entity_index = match archetype.get_entity_index(entity_id) {
                Some(index) => index,
                None => continue,
            };

            if let Some(component_index) = archetype.get_component_index(&type_id) {
                archetype.mark_mutated_single(entity_index, component_index);
                let component = &mut archetype.components[component_index][entity_index];
                return component.downcast_mut::<C>();
            } else {
                return None
            }
        }

        None
    }

    /// Get component
    pub(crate) fn get_component<C: Component>(&self, entity_id: EntityId) -> Option<&C> {
        let type_id = TypeId::of::<C>();
        for archetype in self.archetypes.values() {
            let entity_index = match archetype.get_entity_index(entity_id) {
                Some(index) => index,
                None => continue,
            };

            if let Some(component_index) = archetype.get_component_index(&type_id) {
                let component = &archetype.components[component_index][entity_index];
                return component.downcast_ref::<C>();
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
        assert!(self.archetypes.values().any(|a| a.has_entity(&parent_id)), "Parent entity does not exist");
        assert!(self.archetypes.values().any(|a| a.has_entity(&child_id)), "Child entity does not exist");

        // TODO: Check if child already has a parent and remove it

        if let Some(children) = self.get_component_mut::<Children>(parent_id) {
            children.add(child_id);
        } else {
            self.insert_component(parent_id, Box::new(Children::new(vec![child_id])), true);
        }

        self.insert_component(child_id, Box::new(Parent::new(parent_id)), true);
    }

    /// Remove child from parent's Children component, and remove Parent component from child
    pub(crate) fn remove_child(&mut self, parent_id: EntityId, child_id: EntityId) {
        if let Some(children) = self.get_component_mut::<Children>(parent_id) {
            if children.ids.contains(&child_id) {
                children.remove(child_id);
                let children_len = children.ids.len();
                self.remove_component(child_id, TypeId::of::<Parent>());

                if children_len == 0 {
                    self.remove_component(parent_id, TypeId::of::<Children>());
                }
            }
        }
    }
}
