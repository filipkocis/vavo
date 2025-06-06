use std::{any::{Any, TypeId}, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use crate::query::filter::Filters;

use super::{EntityId, QueryComponentType};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ArchetypeId(u64);

#[derive(Debug)]
pub struct Archetype {
    /// Vec of entity ids in this archetype, where the index corresponds to the entity's component
    /// in `self.components[component_index]`
    entity_ids: Vec<EntityId>,
    /// Stores component type ids and their index in `self.components`
    types: HashMap<TypeId, usize>,
    /// Same layout as components, stores the last tick the component was updated
    ticks: Vec<Vec<u64>>,
    current_tick: *const u64,

    /// Components of the same type are stored together in a row:
    ///
    /// `row 0: A components`
    /// `row 1: B components`
    /// `row 2: ...`
    ///
    ///
    /// Indexes in rows correspond to entity_ids indexes, so that index N in each row represents
    /// the same entity and its components:
    ///
    /// `entity_ids: E1, E2, E3`
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
            self.mark_mutated_single(entity_index, component_index);
            return true
        }
        false
    }

    fn current_tick(&self) -> u64 {
        unsafe { *self.current_tick }
    }

    /// Mark all components in a row of a specific type as changed
    pub(crate) fn mark_mutated(&mut self, type_index: usize) {
        let current_tick = self.current_tick();
        self.ticks[type_index].iter_mut().for_each(|tick| *tick = current_tick);
    }

    /// Mark a specific component of an entity as changed
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

    /// Exposes `self.types` as a sorted vector
    pub fn types_vec(&self) -> Vec<TypeId> {
        let types: Vec<_> = self.types.iter().map(|(k, _)| *k).collect();
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
    pub fn has_types(&self, type_ids: &[TypeId]) -> bool {
        type_ids.iter().all(|type_id| self.has_type(type_id))
    }

    /// Check if all `type_ids` exist in self, no more no less
    pub fn has_types_all(&self, type_ids: &[TypeId]) -> bool {
        self.types.len() == type_ids.len() && self.has_types(type_ids)
    }

    /// Same as [`Self::has_type`] but with generic T type
    pub fn has_t<T: 'static>(self) -> bool {
        self.has_type(&TypeId::of::<T>())
    }

    /// Check if archetype has `entity_id`
    pub fn has_entity(&self, entity_id: &EntityId) -> bool {
        self.entity_ids.contains(entity_id)
    }

    /// Get entity index in `entity_ids` if it exists
    pub fn get_entity_index(&self, entity_id: EntityId) -> Option<usize> {
        self.entity_ids.iter().position(|id| *id == entity_id)
    }

    /// Get component index in `types` if it exists
    pub fn get_component_index(&self, component_id: &TypeId) -> Option<usize> {
        self.types.get(component_id).copied()
    }

    /// Returns hash of sorted types as [`ArchetypeId`]
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
    /// Evaluates filters against this archetype. 
    /// Does **NOT** check `changed` filters, only their type existence in the archetype.
    pub fn matches_filters(&self, filters: &mut Filters) -> bool {
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
    pub fn get_changed_filter_indices(&self, filters: &Filters) -> Vec<Vec<usize>> {
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
        indices[0].iter().all(|&index| self.ticks[index][at] == current_tick)

        // optional Or<T>
        && indices.iter().skip(1).all(|or_indices| 
            or_indices.iter().any(|&index| self.ticks[index][at] == current_tick)
        )
    }
}
