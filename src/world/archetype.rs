use std::{any::{Any, TypeId}, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use crate::query::filter::Filters;

use super::entities::EntityId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ArchetypeId(u64);

#[derive(Debug)]
pub(crate) struct Archetype {
    pub(super) entity_ids: Vec<EntityId>,
    pub(super) types: HashMap<TypeId, usize>,
    /// Same layout as components, stores the last tick the component was updated
    ticks: Vec<Vec<u64>>,
    current_tick: *const u64,

    /// Components of the same type are stored together in a row
    /// ```
    /// vec![
    /// // E: 1  2  3
    ///  vec![A, A, A],
    ///  vec![B, B, B],
    ///  vec![C, C, C],
    /// ]
    /// ```
    pub components: Vec<Vec<Box<dyn Any>>>,
}

impl Archetype {
    pub fn new(types: Vec<TypeId>, current_tick: *const u64) -> Self {
        assert!(!current_tick.is_null(), "Cannot create archetype, current_tick pointer is null");

        let original_len = types.len();
        let types = Self::sort_types(types);
        let types = types.into_iter().enumerate()
            .map(|(i, v)| (v, i))
            .collect::<HashMap<TypeId, usize>>();

        assert!(types.len() == original_len, "Duplicate types in archetype");

        let components = types.iter().map(|_| Vec::new()).collect();
        let ticks = types.iter().map(|_| Vec::new()).collect();

        Self {
            entity_ids: Vec::new(),
            types,
            ticks,
            current_tick,
            components,
        }
    }

    /// Insert new entity
    pub(super) fn insert_entity(&mut self, entity_id: EntityId, components: Vec<Box<dyn Any>>) {
        self.entity_ids.push(entity_id);

        let components = components.into_iter()
            .map(|v| ((*v).type_id(), v))
            .collect::<Vec<_>>();

        let component_types = components.iter().map(|(t, _)| *t).collect::<Vec<_>>();
        assert!(self.has_types_all(&component_types), "Component types mismatch with archetype types");

        for (type_id, component) in components {
            let component_index = self.types[&type_id];
            self.components[component_index].push(component);
            let current_tick = self.current_tick();
            self.ticks[component_index].push(current_tick.max(1)); // 0 is during startup
        }

        assert!(
            self.components.iter().all(|row| row.len() == self.entity_ids.len()), 
            "Specific components length mismatch with entity IDs length"
        );
        assert!(
            self.ticks.iter().all(|row| row.len() == self.entity_ids.len()), 
            "Specific ticks length mismatch with entity IDs length"
        );
        assert!(
            self.components.len() == self.types.len(),
            "Components length mismatch with types length"
        );
        assert!(
            self.ticks.len() == self.types.len(),
            "Ticks length mismatch with types length"
        );
    }

    /// Remove entity, returns removed components or None if entity_id doesn't exist
    pub(super) fn remove_entity(&mut self, entity_id: EntityId) -> Option<Vec<Box<dyn Any>>> {
        if let Some(index) = self.entity_ids.iter().position(|id| *id == entity_id) {
            self.entity_ids.remove(index);
            self.ticks.iter_mut().for_each(|ticks| { ticks.remove(index); });

            let mut removed = Vec::with_capacity(self.components.len());
            for components in self.components.iter_mut() {
                removed.push(components.remove(index))
            }

            return Some(removed);
        }

        None
    }

    /// Update component, returns true if successful
    pub(super) fn update_component(&mut self, entity_id: EntityId, component: Box<dyn Any>) -> bool {
        if let Some(entity_index) = self.entity_ids.iter().position(|id| *id == entity_id) {
            let type_id = (*component).type_id();
            let component_index = self.types[&type_id];
            self.components[component_index][entity_index] = component;
            self.ticks[component_index][entity_index] = self.current_tick();
            return true
        }
        false
    }

    fn current_tick(&self) -> u64 {
        unsafe { *self.current_tick }
    }

    pub(crate) fn mark_mutated(&mut self, type_index: usize) {
        let current_tick = self.current_tick();
        self.ticks[type_index].iter_mut().for_each(|tick| *tick = current_tick);
    }

    pub(crate) fn mark_mutated_single(&mut self, entity_index: usize, type_index: usize) {
        self.ticks[type_index][entity_index] = self.current_tick();
    }

    pub(crate) fn components_at_mut(&mut self, index: usize) -> *mut Vec<Box<dyn Any>> {
        &mut self.components[index]
    }

    /// Returns sorted types
    pub fn sort_types(mut types: Vec<TypeId>) -> Vec<TypeId> {
        types.sort_by(|a, b| a.cmp(b));
        types
    }

    /// Amount of entities in this archetype
    pub fn len(&self) -> usize {
        self.entity_ids.len()
    }

    /// Exposes types hashmap
    pub fn types(&self) -> &HashMap<TypeId, usize> {
        &self.types
    }

    /// Exposes sorted types as vector
    pub fn types_vec(&self) -> Vec<TypeId> {
        let types: Vec<_> = self.types.iter().map(|(k, _)| *k).collect();
        Self::sort_types(types)
    }

    /// Check if type_id exists in self
    pub fn has_type(&self, type_id: &TypeId) -> bool {
        self.types.contains_key(type_id)
    }

    /// Check if all type_ids exist in self
    pub fn has_types(&self, type_ids: &[TypeId]) -> bool {
        type_ids.iter().all(|type_id| self.has_type(type_id))
    }

    /// Check if all type_ids exist in self, no more no less
    pub fn has_types_all(&self, type_ids: &[TypeId]) -> bool {
        self.types.len() == type_ids.len() && self.has_types(type_ids)
    }

    /// Same as has_type but with generic T type
    pub fn has_t<T: 'static>(self) -> bool {
        self.has_type(&TypeId::of::<T>())
    }

    /// Check if archetype has entity_id
    pub fn has_entity(&self, entity_id: EntityId) -> bool {
        self.entity_ids.contains(&entity_id)
    }

    /// Get entity index in entity_ids if it exists
    pub fn get_entity_index(&self, entity_id: EntityId) -> Option<usize> {
        self.entity_ids.iter().position(|id| *id == entity_id)
    }

    /// Returns hash of sorted types
    pub(super) fn hash_types(types: Vec<TypeId>) -> ArchetypeId {
        let mut hasher = DefaultHasher::new();
        let types = Self::sort_types(types);

        for type_id in types {
            type_id.hash(&mut hasher);
        }

        let hash = hasher.finish();
        ArchetypeId(hash)
    }
}

impl Archetype {
    pub fn matches_filters(&self, filters: &Filters) -> bool {
        if filters.empty {
            return true
        }

        self.match_filter_changed(&filters) &&
        self.match_filter_with(&filters) &&
        self.match_filter_without(&filters)
    }

    /// Returns indices of requested changed fields in this archetype
    ///
    /// # Panics
    /// Panics if type_id in filters.changed is not found in archetype
    pub fn get_changed_filter_indices(&self, filters: &Filters) -> Vec<usize> {
        filters.changed.iter().filter_map(|type_id| 
            Some(*self.types.get(type_id)
                .expect("type_id in filters.changed not found in archetype"))
        ).collect()
    }

    /// Checks if requested fields (indices) are marked as changed in entities[at]
    ///
    /// # Note
    /// To get the correct indices call `archetype.get_changed_filter_indices(filters)`
    pub fn check_changed_fields(&self, at: usize, indices: &[usize]) -> bool {
        if indices.is_empty() {
            return true
        }

        indices.iter().all(|&index| {
            self.ticks[index][at] == self.current_tick()
        })
    }

    fn match_filter_changed(&self, filters: &Filters) -> bool {
        filters.changed.iter().all(|type_id| self.has_type(type_id))
    }

    fn match_filter_with(&self, filters: &Filters) -> bool {
        filters.with.iter().all(|type_id| self.has_type(type_id))
    }

    fn match_filter_without(&self, filters: &Filters) -> bool {
        filters.without.iter().all(|type_id| !self.has_type(type_id)) 
    }
}
