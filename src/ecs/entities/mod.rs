pub mod archetype;
pub mod components;
pub mod relation;
pub mod tracking;

pub use components::Component;
use components::ComponentInfoPtr;

use std::{any::TypeId, collections::HashMap, hash::Hash, mem::ManuallyDrop};

use crate::ecs::entities::{archetype::TypedComponentData, tracking::EntityTracking};
use crate::macros::{Component, Reflect};
use crate::query::{filter::Filters, QueryComponentType};

use archetype::{Archetype, ArchetypeId};
use relation::{Children, Parent};

use super::{ptr::OwnedPtr, tick::Tick};

/// Unique identifier for an [entity](Entities) in a [`World`](crate::ecs::world::World).
/// Consists of an `index` and a `generation` to avoid reusing IDs of despawned entities.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Component, Reflect)]
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
        Self { index, generation }
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

/// Entity store, manages archetypes and all their entities (components) in the `world`
#[derive(Debug)]
pub struct Entities {
    /// Not public since entity creation and despawning is done via commands, which if not applied
    /// by the user would lead to tracked entities which do not exist
    pub(crate) tracking: EntityTracking,
    /// Holds all archetypes in the world by their unique id
    pub(crate) archetypes: HashMap<ArchetypeId, Archetype>,
    /// Pointer to current tick in the world, used for component change tracking
    current_tick: *const Tick,
    /// Info pointer for EntityId component insertion
    entity_info: ComponentInfoPtr,
}

impl Default for Entities {
    fn default() -> Self {
        Self {
            tracking: EntityTracking::new(),
            archetypes: HashMap::new(),
            current_tick: std::ptr::null(),
            entity_info: ComponentInfoPtr::null(),
        }
    }
}

// TODO: Implement the correct removal and transfer of ticks, not just components

impl Entities {
    /// Create new `Entities` manager with uninitialized tick pointer
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns current tick
    #[inline]
    pub fn tick(&self) -> Tick {
        debug_assert!(
            !self.current_tick.is_null(),
            "Entities tick pointer is null. Did you forget to call `initialize`?",
        );
        unsafe { *self.current_tick }
    }

    /// Returns entity info
    #[inline]
    fn entity_info(&self) -> ComponentInfoPtr {
        debug_assert!(
            !self.entity_info.is_null(),
            "Entities EntityId component info pointer is null. Did you forget to call `initialize`?",
        );
        self.entity_info
    }

    /// Exposes archetyeps
    #[inline]
    pub fn archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.values()
    }

    // / Initialize tick pointer and entity info, necessary for entity creation. Done in
    /// [`World`](crate::prelude::World) initialization.
    #[inline]
    pub fn initialize(&mut self, current_tick: *const Tick, entity_info: ComponentInfoPtr) {
        self.current_tick = current_tick;
        self.entity_info = entity_info;
    }

    /// Returns archetypes with matching [`query types`](QueryComponentType) and filters, and component indices for
    /// `changed` filters acquired from [`Archetype::get_changed_filter_indices`]
    pub(crate) fn archetypes_filtered<'a>(
        &'a mut self,
        type_ids: &'a [QueryComponentType],
        filters: &'a mut Filters,
    ) -> impl Iterator<Item = (&'a mut Archetype, Vec<Vec<usize>>)> {
        self.archetypes.values_mut().filter_map(|archetype| {
            archetype
                .filtered(type_ids, filters)
                .map(|indices| (archetype, indices))
        })
    }

    /// Spawn new entity with components
    ///
    /// # Panic
    /// Panics if components contain EntityId
    pub(crate) fn spawn_entity(
        &mut self,
        entity_id: EntityId,
        components: Vec<(ComponentInfoPtr, OwnedPtr)>,
    ) {
        assert!(
            !components
                .iter()
                .any(|(info, _)| info.as_ref().type_id == TypeId::of::<EntityId>()),
            "Cannot insert EntityId as a component"
        );

        let tick = self.tick();

        // Build typed components
        let mut components = components
            .into_iter()
            .map(|(info, data)| TypedComponentData::from_parts(info, data, tick, tick))
            .collect::<Vec<_>>();

        // Insert builtin EntityId component
        let mut entity_id_cpy = ManuallyDrop::new(entity_id);
        // Safety: entity is copied because its just on the stack
        let entity_id_ptr = unsafe { OwnedPtr::new_ref(&mut entity_id_cpy) };
        components.push(TypedComponentData::from_parts(
            self.entity_info(),
            entity_id_ptr,
            tick,
            tick,
        ));

        // Sort components by type id
        components.sort_by_key(|component| component.info.as_ref().type_id);

        // Safety: componetns are correct and sorted
        let archetype_id = unsafe { Archetype::hash_sorted_components(&mut components) };

        // Get or create archetype
        let archetype = self.archetypes.entry(archetype_id).or_insert_with(|| {
            let infos = components.iter().map(|component| component.info).collect();
            Archetype::new(archetype_id, infos)
        });

        // Safety: components are correct and sorted
        let location = unsafe { archetype.insert_entity(entity_id, components) };

        // Track entity location
        self.tracking.set_location(entity_id, location);
    }

    /// Despawn entity and break all relations
    pub(crate) fn despawn_entity(&mut self, entity_id: EntityId) {
        // Remove link to parent
        if let Some(parent) = self.get_component::<Parent>(entity_id) {
            self.remove_child(parent.id, entity_id);
        }
        // Remove links to children
        if let Some(children) = self.get_component::<Children>(entity_id) {
            for child_id in children.ids.clone() {
                self.remove_child(entity_id, child_id);
            }
        }

        // Get entity location
        let Some(location) = self.tracking.get_location(entity_id) else {
            return;
        };

        let id = location.archetype_id();
        let archetype = self
            .archetypes
            .get_mut(&id)
            .expect("archetype should exist");

        // Remove entity
        let removed = archetype.remove_entity(entity_id, location);
        for component in removed.components {
            component.drop();
        }
        self.tracking.remove_entity(entity_id);

        // Update swapped entity location
        if let Some(swapped) = removed.swapped {
            self.tracking.set_location(swapped, location);
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
    pub(crate) fn insert_component(
        &mut self,
        entity_id: EntityId,
        component: OwnedPtr,
        info: ComponentInfoPtr,
        replace: bool,
    ) {
        let tick = self.tick();
        let type_id = info.as_ref().type_id;
        let archetypes_ptr = &mut self.archetypes as *mut HashMap<_, _>;
        assert_ne!(
            type_id,
            TypeId::of::<EntityId>(),
            "Cannot insert EntityId as a component"
        );

        // Get entity location
        let Some(location) = self.tracking.get_location(entity_id) else {
            info.drop(component);
            return;
        };

        // Get current archetype
        let id = location.archetype_id();
        let archetype = self
            .archetypes
            .get_mut(&id)
            .expect("archetype should exist");

        // If component type already exists, replace it or drop the new one
        if archetype.has_type(&type_id) {
            if replace {
                let component = TypedComponentData::from_parts(info, component, tick, tick);
                archetype.set_component(entity_id, location, component);
            } else {
                info.drop(component);
            }
            return;
        }

        // Remove entity from archetype and add new component
        let mut removed = archetype.remove_entity(entity_id, location);
        let new_component = TypedComponentData::from_parts(info, component, tick, tick);
        removed.components.push(new_component);

        // Sort components by type id
        removed
            .components
            .sort_by_key(|component| component.info.as_ref().type_id);

        // Update swapped entity location
        if let Some(swapped) = removed.swapped {
            self.tracking.set_location(swapped, location);
        }

        // Safety: components are correct and sorted
        let new_id = unsafe { Archetype::hash_sorted_components(&mut removed.components) };

        // Safety: since `removed` references archetype, we need to do another mut borrow
        // which is safe here because we are accessing a different archetype
        let archetypes = unsafe { &mut *archetypes_ptr };

        // Insert entity into new archetype
        let new_archetype = archetypes.entry(new_id).or_insert_with(|| {
            let infos = removed
                .components
                .iter()
                .map(|component| component.info)
                .collect();
            Archetype::new(new_id, infos)
        });

        // Safety: components are correct and sorted
        let new_location = unsafe { new_archetype.insert_entity(entity_id, removed.components) };

        // Update entity location
        self.tracking.set_location(entity_id, new_location);
    }

    /// Remove component
    ///
    /// # Panics
    /// Panics if type_id is EntityId
    pub(crate) fn remove_component(&mut self, entity_id: EntityId, type_id: TypeId) {
        let archetypes_ptr = &mut self.archetypes as *mut HashMap<_, _>;
        assert_ne!(
            type_id,
            TypeId::of::<EntityId>(),
            "Cannot remove builtin EntityId component"
        );

        // Get entity location
        let Some(location) = self.tracking.get_location(entity_id) else {
            return;
        };

        let id = location.archetype_id();
        let archetype = self
            .archetypes
            .get_mut(&id)
            .expect("archetype should exist");

        // Get component type index
        let component_index = archetype.component_index(&type_id);

        // Remove entity from archetype and remove component
        let mut removed = archetype.remove_entity(entity_id, location);
        let removed_data = removed.components.remove(component_index);
        removed_data.drop();

        // Update swapped entity location
        if let Some(swapped) = removed.swapped {
            self.tracking.set_location(swapped, location);
        }

        // Safety: components are correct and sorted because removal preserves order
        let new_id = unsafe { Archetype::hash_sorted_components(&mut removed.components) };

        // Safety: since `removed` references archetype, we need to do another mut borrow
        // which is safe here because we are accessing a different archetype
        let archetypes = unsafe { &mut *archetypes_ptr };

        // Insert entity into new archetype
        let new_archetype = archetypes.entry(new_id).or_insert_with(|| {
            let infos = removed
                .components
                .iter()
                .map(|component| component.info)
                .collect();
            Archetype::new(new_id, infos)
        });

        // Safety: components are correct and sorted
        let new_location = unsafe { new_archetype.insert_entity(entity_id, removed.components) };

        // Update entity location
        self.tracking.set_location(entity_id, new_location);
    }

    /// Get component mutably if it exists, marking it as changed
    pub(crate) fn get_component_mut<C: Component>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<&mut C> {
        let current_tick = self.tick();

        // Get entity location
        let location = self.tracking.get_location(entity_id)?;
        let entity_index = location.index();
        let id = location.archetype_id();
        let archetype = self
            .archetypes
            .get_mut(&id)
            .expect("archetype should exist");

        // Get components by type index
        let component_index = archetype.try_component_index(&TypeId::of::<C>())?;
        let components = &mut archetype.components[component_index];

        // Mark component as changed
        components.set_changed_at(entity_index, current_tick);

        // Get component mutable reference
        let component = unsafe {
            // Safety: entity existence is guaranteed by tracking
            components
                .get_untyped_lt(entity_index)
                .as_ptr()
                .cast::<C>()
                .as_mut()
        };

        Some(component)
    }

    /// Get component if it exists
    pub(crate) fn get_component<C: Component>(&self, entity_id: EntityId) -> Option<&C> {
        // Get entity location
        let location = self.tracking.get_location(entity_id)?;
        let entity_index = location.index();
        let id = location.archetype_id();
        let archetype = self.archetypes.get(&id).expect("archetype should exist");

        // Get components by type index
        let component_index = archetype.try_component_index(&TypeId::of::<C>())?;
        let components = &archetype.components[component_index];

        // Get component reference
        let component = unsafe {
            // Safety: entity existence is guaranteed by tracking
            components
                .get_untyped_lt(entity_index)
                .as_ptr()
                .cast::<C>()
                .as_ref()
        };

        Some(component)
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
        children_info: ComponentInfoPtr,
    ) {
        assert_ne!(
            parent_id, child_id,
            "Parent and child cannot be the same entity"
        );
        assert!(
            self.tracking.get_location(parent_id).is_some(),
            "Parent entity does not exist"
        );
        assert!(
            self.tracking.get_location(child_id).is_some(),
            "Child entity does not exist"
        );

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

    /// Breaks the relation link between parent and child.
    /// Remove child from parent's Children component, and remove Parent component from child.
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
