pub mod archetype;
pub mod relation;
pub mod components;

pub use components::Component;
use components::ComponentInfoPtr;

use std::{any::TypeId, collections::HashMap, hash::Hash, mem::ManuallyDrop};

use crate::query::{filter::Filters, QueryComponentType};
use crate::macros::{Component, Reflect};

use archetype::{Archetype, ArchetypeId};
use relation::{Children, Parent};

use super::{ptr::OwnedPtr, tick::Tick};

/// Unique identifier for an [entity](Entities) in a [`World`](crate::ecs::world::World).
/// Consists of an `index` and a `generation` to avoid reusing IDs of despawned entities.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[derive(Component, Reflect)]
pub struct EntityId {
    /// Index of the entity, serves as the main identifier and is reused after despawning an
    /// entity. It's used as an index in the entities storage.
    index: u32,
    /// Generation of the entity, incremented every time an entity with the same index is reused.
    generation: u32,
}

impl EntityId {
    /// Create new EntityId from index and generation
    #[inline]
    pub fn new(index: u32, generation: u32) -> Self {
        Self {index, generation }
    }

    /// Returns the index of the id
    #[inline]
    pub fn index(self) -> u32 {
        self.index
    }

    /// Returns the generation of the id
    #[inline]
    pub fn generation(self) -> u32 {
        self.generation
    }

    /// Returns a u64 representation of the id
    /// Lower 32 bits are index, upper 32 bits are generation
    #[inline]
    pub fn to_bits(self) -> u64 {
        (self.index as u64) | ((self.generation as u64) << 32) 
    }

    /// Create a new id from a u64 representation
    /// Lower 32 bits are index, upper 32 bits are generation
    #[inline]
    pub fn from_bits(bits: u64) -> Self {
        let index = (bits & 0xFFFFFFFF) as u32;
        let generation = ((bits >> 32) & 0xFFFFFFFF) as u32;
        Self { index, generation }
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
            next_entity_id: EntityId::new(0, 0),
            archetypes: HashMap::new(),
            current_tick: std::ptr::null(),
        }
    }

    #[inline]
    /// Returns current tick
    pub fn tick(&self) -> Tick {
        unsafe { *self.current_tick }
    }

    /// Exposes archetyeps
    pub fn archetypes(&self) -> impl Iterator<Item = &Archetype> { 
        self.archetypes.values()
    }

    /// Initialize tick pointer, necessary for entity creation. Done in
    /// [`World`](crate::prelude::World) initialization.
    pub fn initialize_tick(&mut self, current_tick: *const Tick) {
        self.current_tick = current_tick
    }

    /// Step next entity ID counter
    /// Returns new entity ID
    fn step_entity_id(&mut self) -> EntityId {
        self.next_entity_id.index += 1;
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
    pub fn next_entity_id(&self) -> EntityId {
        self.next_entity_id
    }

    /// Spawn new entity with components
    ///
    /// # Panic
    /// Panics if components contain EntityId
    pub(crate) fn spawn_entity(
        &mut self, 
        entity_id: EntityId, 
        components: Vec<(ComponentInfoPtr, OwnedPtr)>, 
        entity_info: ComponentInfoPtr
    ) {
        let entity_id_type = TypeId::of::<EntityId>();
        assert!(
            !components.iter().any(|(info, _)| info.as_ref().type_id == entity_id_type),
            "Cannot insert EntityId as a component"
        );

        let tick = self.tick();
        let mut components = components.into_iter().map(|(info, ptr)| (info, ptr, tick, tick)).collect::<Vec<_>>();
        let mut entity_id_cpy = ManuallyDrop::new(entity_id);

        // Safety: entity is copied because its just on the stack
        let entity_id_ptr = unsafe { OwnedPtr::new_ref(&mut entity_id_cpy) }; 
        components.push((
            entity_info,
            entity_id_ptr,
            tick,
            tick,
        ));
        let infos = components.iter().map(|(info, ..)| *info).collect::<Vec<_>>();
        let archetype_id = Archetype::hash_types(infos.clone()); 

        assert!(
            self.next_entity_id == entity_id, 
            "Entity ID mismatch with next entity ID (id {:?} != next {:?})", 
            entity_id, self.next_entity_id
        );
        self.step_entity_id();

        self.archetypes.entry(archetype_id)
            .or_insert_with(|| Archetype::new(infos, self.current_tick))
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
            if let Some(removed) = archetype.remove_entity(entity_id) {
                for (info, component, ..) in removed {
                    info.drop(component)
                }
            }

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
    pub(crate) fn insert_component(&mut self, entity_id: EntityId, component: OwnedPtr, info: ComponentInfoPtr, replace: bool) {
        let current_tick = self.tick();
        let type_id = info.as_ref().type_id;
        let archetypes_ptr = &mut self.archetypes as *mut HashMap<_, _>;
        assert_ne!(type_id, TypeId::of::<EntityId>(), "Cannot insert EntityId as a component");

        let mut emptied_archetype = None;
        if let Some((id, archetype)) = self.archetypes.iter_mut().find(|(_, a)| a.has_entity(&entity_id)) {
            if archetype.has_type(&type_id) {
                if replace {
                    assert!(archetype.set_component(entity_id, component, info.as_ref().type_id), "Failed to set component");
                } else {
                    info.drop(component);
                }
                return;
            }

            if archetype.len() == 1 {
                emptied_archetype = Some(*id);
            }

            let mut infos = archetype.types_vec();
            infos.push(info);

            let mut old_components = archetype.remove_entity(entity_id).expect("entity_id should exist in archetype");
            old_components.push((info, component, current_tick, current_tick));

            let archetype_id = Archetype::hash_types(infos.clone());
            // Safety: since old_components reference archetype, we need to do another mut borrow
            // which is safe here because the reference is from `remove_entity`
            unsafe { &mut *archetypes_ptr }.entry(archetype_id)
                .or_insert_with(|| Archetype::new(infos, self.current_tick))
                .insert_entity(entity_id, old_components);
        } else {
            info.drop(component);
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
        let archetypes_ptr = &mut self.archetypes as *mut HashMap<_, _>;
        assert_ne!(type_id, TypeId::of::<EntityId>(), "Cannot remove builtin EntityId component");

        let mut emptied_archetype = None;
        if let Some((id, archetype)) = self.archetypes.iter_mut().find(|(_, a)| a.has_entity(&entity_id)) {
            let Some(component_index) = archetype.get_component_index(&type_id) else {
                return;
            };

            if archetype.len() == 1 {
                emptied_archetype = Some(*id);
            }

            let mut types = archetype.types_vec();
            types.remove(component_index);

            let mut old_components = archetype.remove_entity(entity_id).expect("entity_id should exist in archetype");
            let removed = old_components.remove(component_index);
            removed.0.drop(removed.1);

            let archetype_id = Archetype::hash_types(types.clone());
             // Safety: since old_components reference archetype, we need to do another mut borrow
            // which is safe here because the reference is from `remove_entity`
            unsafe { &mut *archetypes_ptr }.entry(archetype_id)
                .or_insert_with(|| Archetype::new(types, self.current_tick))
                .insert_entity(entity_id, old_components);
        }

        if let Some(id) = emptied_archetype {
            self.archetypes.remove(&id);
        }
    }

    /// Get component mutably
    pub(crate) fn get_component_mut<C: Component>(&mut self, entity_id: EntityId) -> Option<&mut C> {
        let current_tick = self.tick();
        let type_id = TypeId::of::<C>();
        for archetype in self.archetypes.values_mut() {
            let entity_index = match archetype.get_entity_index(entity_id) {
                Some(index) => index,
                None => continue,
            };

            if let Some(component_index) = archetype.get_component_index(&type_id) {
                let comp = &mut archetype.components[component_index];
                comp.set_changed_at(entity_index, current_tick);
                let comp = unsafe { comp.get_untyped_lt(entity_index).as_ptr().cast::<C>().as_mut() };
                return Some(comp)
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
                let comp = &archetype.components[component_index];
                let comp = unsafe { comp.get_untyped_lt(entity_index).as_ptr().cast::<C>().as_ref() };
                return Some(comp)
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
    pub(crate) fn add_child(
        &mut self,
        parent_id: EntityId,
        child_id: EntityId,
        parent_info: ComponentInfoPtr,
        children_info: ComponentInfoPtr
    ) {
        assert_ne!(parent_id, child_id, "Parent and child cannot be the same entity");
        assert!(self.archetypes.values().any(|a| a.has_entity(&parent_id)), "Parent entity does not exist");
        assert!(self.archetypes.values().any(|a| a.has_entity(&child_id)), "Child entity does not exist");

        // TODO: Check if child already has a parent and remove it

        if let Some(children) = self.get_component_mut::<Children>(parent_id) {
            children.add(child_id);
        } else {
            let children = Children::new(vec![child_id]);
            let mut children = ManuallyDrop::new(children);
            let ptr = unsafe { OwnedPtr::new_ref(&mut children) }; // safety: children not used after this
            self.insert_component(parent_id, ptr, children_info, true);
        }

        let parent = Parent::new(parent_id);
        let mut parent = ManuallyDrop::new(parent);
        let ptr = unsafe { OwnedPtr::new_ref(&mut parent) }; // safety: parent not used after this
        self.insert_component(child_id, ptr, parent_info, true);
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
