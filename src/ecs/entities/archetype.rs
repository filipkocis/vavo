use std::{any::{Any, TypeId}, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use crate::{ecs::ptr::UntypedPtr, prelude::Tick, query::filter::Filters};

use super::{components::{ComponentInfoPtr, ComponentsData}, EntityId, QueryComponentType};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ArchetypeId(u64);

#[derive(Debug)]
pub struct Archetype {
    /// Stores component type ids and their index in `self.components`
    types: HashMap<TypeId, (usize, ComponentInfoPtr)>,
    current_tick: *const Tick,
    /// Entity ids in this archetype, `self.entity_ids[entity]` has components at
    /// `self.components[N][entity]` `
    entity_ids: Vec<EntityId>,
    /// Component buckets
    pub components: Vec<ComponentsData>,
}

impl Archetype {
    /// Create new archetype with `types`.
    pub fn new(infos: Vec<ComponentInfoPtr>, current_tick: *const Tick) -> Self {
        assert!(!current_tick.is_null(), "Cannot create archetype, current_tick pointer is null");

        let original_len = infos.len();
        let sorted_types = Self::sort_types(infos);

        let components = sorted_types.iter()
            .map(|t| ComponentsData::new(*t)).collect();

        let types = sorted_types.into_iter().enumerate()
            .map(|(i, v)| (v.as_ref().type_id, (i, v)))
            .collect::<HashMap<_, _>>();

        assert!(types.len() == original_len, "Duplicate types in archetype");

        Self {
            entity_ids: Vec::new(),
            types,
            current_tick,
            components,
        }
    }

    /// Insert new entity
    pub(super) fn insert_entity(&mut self, entity_id: EntityId, components: Vec<(ComponentInfoPtr, UntypedPtr, Tick, Tick)>) {
        self.entity_ids.push(entity_id);

        // let components = components.into_iter()
        //     .map(|v| ((*v).type_id(), v))
        //     .collect::<Vec<_>>();

        let component_types = components.iter().map(|(t, ..)| t.as_ref().type_id).collect::<Vec<_>>();
        assert!(self.has_types_all(&component_types), "Component types mismatch with archetype types");

        for (info, component, changed_at, added_at) in components {
            let component_index = self.types[&info.as_ref().type_id].0;
            self.components[component_index].insert(component, changed_at, added_at);

            // TODO: this is no longer valid syntax, so find a place where to use tick+=1 or
            // tick.max(1)
            // let current_tick = self.current_tick();
            // self.ticks[component_index].push(current_tick.max(1)); // 0 is during startup
        }

        debug_assert!(
            self.components.iter().all(|row| row.len() == self.entity_ids.len()), 
            "Specific components length mismatch with entity IDs length"
        );
        debug_assert!(
            self.components.len() == self.types.len(),
            "Components length mismatch with types length"
        );
    }

    /// Remove entity, returns removed components with `(changed_at, added_at)` ticks or None if entity_id doesn't exist
    pub(super) fn remove_entity(&mut self, entity_id: EntityId) -> Option<Vec<(ComponentInfoPtr, UntypedPtr, Tick, Tick)>> {
        if let Some(index) = self.entity_ids.iter().position(|id| *id == entity_id) {
            self.entity_ids.remove(index);

            let mut removed = Vec::with_capacity(self.components.len());
            for components_data in &mut self.components {
                // TODO: create a self.types_vec copy so we dont have to use a hashmap here (if it
                // helps the performance)
                let info_ptr = self.types[&components_data.get_type_id()].1; 
                let rem = components_data.remove(index);
                removed.push((
                    info_ptr,
                    rem.0,
                    rem.1,
                    rem.2,
                ))
            }

            return Some(removed);
        }

        None
    }

    /// Sets component to a new value, returns true if successful
    pub(super) fn set_component(&mut self, entity_id: EntityId, component: UntypedPtr, type_id: TypeId) -> bool {
        let current_tick = self.current_tick();
        if let Some(entity_index) = self.entity_ids.iter().position(|id| *id == entity_id) {
            let component_index = self.types[&type_id].0;
            self.components[component_index].set(entity_index, component, current_tick);
            return true
        } else {
            self.types[&type_id].1.drop(component);
            return false
        }
    }

    #[inline]
    fn current_tick(&self) -> Tick {
        unsafe { *self.current_tick }
    }

    #[inline]
    /// Amount of entities in this archetype
    pub fn len(&self) -> usize {
        self.entity_ids.len()
    }

    /// Mark all components in a row of a specific type as changed
    pub(crate) fn mark_mutated(&mut self, _type_index: usize) {
        todo!("remove this function once query is rewritten")
        // let current_tick = self.current_tick();
        // self.ticks[type_index].iter_mut().for_each(|tick| *tick = current_tick);
    }

    /// Mark a specific component of an entity as changed
    pub(crate) fn mark_mutated_single(&mut self, _entity_index: usize, _type_index: usize) {
        todo!("remove this function once query is rewritten")
        // self.ticks[type_index][entity_index] = self.current_tick();
    }

    pub(crate) fn components_at_mut(&mut self, _index: usize) -> *mut Vec<Box<dyn Any>> {
        todo!("remove this function once query is rewritten")
        // &mut self.components[index]
    }

    /// Returns sorted types
    fn sort_types(mut types: Vec<ComponentInfoPtr>) -> Vec<ComponentInfoPtr> {
        types.sort_by(|a, b| a.as_ref().type_id.cmp(&b.as_ref().type_id));
        types.to_vec()
    }

    /// Exposes `self.types` as a sorted vector
    pub fn types_vec(&self) -> Vec<ComponentInfoPtr> {
        let types: Vec<_> = self.types.iter().map(|(_, (_, v))| *v).collect();
        Self::sort_types(types)
    }

    /// Check if `type_id` exists in self
    pub fn has_type(&self, type_id: &TypeId) -> bool {
        self.types.contains_key(type_id)
    }

    /// Check if all [`QueryComponentType::Normal`] types exist in self
    pub(crate) fn has_query_types(&self, type_ids: &[QueryComponentType]) -> bool {
        type_ids.iter().all(|type_id| {
            match type_id {
                QueryComponentType::Normal(type_id) => self.has_type(type_id),
                QueryComponentType::Option(_) => true,
            }
        })
    }

    /// Check if all `type_ids` exist in self
    #[inline]
    pub fn has_types(&self, type_ids: &[TypeId]) -> bool {
        type_ids.iter().all(|type_id| self.has_type(type_id))
    }

    /// Check if all `type_ids` exist in self, no more no less
    #[inline]
    pub fn has_types_all(&self, type_ids: &[TypeId]) -> bool {
        self.types.len() == type_ids.len() && self.has_types(type_ids)
    }

    /// Check if archetype has `entity_id`
    #[inline]
    pub fn has_entity(&self, entity_id: &EntityId) -> bool {
        self.entity_ids.contains(entity_id)
    }

    /// Get entity index in `entity_ids` if it exists
    #[inline]
    pub fn get_entity_index(&self, entity_id: EntityId) -> Option<usize> {
        self.entity_ids.iter().position(|id| *id == entity_id)
    }

    /// Get component index in `types` if it exists
    #[inline]
    pub fn get_component_index(&self, component_id: &TypeId) -> Option<usize> {
        self.types.get(component_id).map(|(index, _)| *index)
    }

    /// Returns hash of sorted types as [`ArchetypeId`]
    pub(super) fn hash_types(types: Vec<ComponentInfoPtr>) -> ArchetypeId {
        let mut hasher = DefaultHasher::new();
        let types = Self::sort_types(types);

        for comp_info in types {
            comp_info.as_ref().type_id.hash(&mut hasher);
        }

        let hash = hasher.finish();
        ArchetypeId(hash)
    }
}

impl Archetype {
    /// Evaluates filters against this archetype. 
    /// Does **NOT** check `changed` filters, only their type existence in the archetype.
    pub(crate) fn matches_filters(&self, filters: &mut Filters) -> bool {
        if filters.empty {
            return true
        }

        self.has_types(&filters.changed) &&
        self.has_types(&filters.with) &&
        filters.without.iter().all(|type_id| !self.has_type(type_id)) && 
        filters.or.iter_mut().all(|filters| self.matches_filters_any(filters))
    }

    /// Returns true if any of the filters evaluate to true
    fn matches_filters_any(&self, filters: &mut Filters) -> bool {
        assert!(filters.or.is_empty(), "Nested OR filters are not supported");

        if filters.empty {
            return true
        }

        filters.matches_existence =
            filters.with.iter().any(|type_id| self.has_type(type_id)) ||
            filters.without.iter().any(|type_id| !self.has_type(type_id));

        filters.matches_existence ||
        filters.changed.iter().any(|type_id| self.has_type(type_id))
    }

    /// Returns indices of requested changed fields in this archetype, where first vec is from
    /// `filters.changed` and the rest (optional) are from `filters.or[n].changed`.
    ///
    /// # Panics
    /// Panics if type_id in `filters.changed` is not found in archetype
    pub(crate) fn get_changed_filter_indices(&self, filters: &Filters) -> Vec<Vec<usize>> {
        let mut result = Vec::with_capacity(1); 

        let base = filters.changed.iter().map(|component_id|
            self.get_component_index(component_id).expect("Component from filters.changed not found in archetype")
        ).collect::<Vec<_>>();
        result.push(base);

        for or_filters in &filters.or {
            if or_filters.matches_existence {
                continue
            }

            let or_indices = or_filters.changed.iter().filter_map(|component_id|
                self.get_component_index(component_id)
            ).collect::<Vec<_>>();

            if or_indices.is_empty() {
                if or_filters.with.len() + or_filters.without.len() == 0 {
                    panic!("Or<T> filter only contains changed filters, but none of the types are found in archetype");
                } else {
                    panic!("Or<T> filter doesn't match existence filters, and none of the Changed<T> types are found in archetype");
                } 
            }

            result.push(or_indices);
        }

        result
    }

    /// Checks if requested fields (indices) are marked as changed in entities[at]
    ///
    /// # Note
    /// To get the correct indices call `archetype.get_changed_filter_indices(filters)`
    pub fn check_changed_fields(&self, at: usize, indices: &[Vec<usize>]) -> bool {
        if indices.len() == 1 && indices[0].is_empty() {
            return true
        }

        let current_tick = self.current_tick();

        // base filter
        indices[0].iter().all(|&index| self.components[index].has_changed(at, current_tick))

        // optional Or<T>
        && indices.iter().skip(1).all(|or_indices| 
            or_indices.iter().any(|&index| self.components[index].has_changed(at, current_tick))
        )
    }
}
