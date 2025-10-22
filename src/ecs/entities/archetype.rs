use std::{
    any::TypeId,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    ecs::{
        entities::{components::UntypedComponentData, tracking::EntityLocation},
        ptr::OwnedPtr,
    },
    prelude::Tick,
    query::filter::Filters,
};

use super::{
    components::{ComponentInfoPtr, ComponentsData},
    EntityId, QueryComponentType,
};

/// Holds owned component data with its type information. Either from removed
/// entity in [`Archetype`] or when moving components between archetypes.
///
/// # Note
/// The data must be dropped manully by calling [`Self::drop`]
#[derive(Debug)]
pub(crate) struct TypedComponentData<'a> {
    pub info: ComponentInfoPtr,
    pub data: UntypedComponentData<'a>,
}

impl<'a> TypedComponentData<'a> {
    pub fn new(info: ComponentInfoPtr, data: UntypedComponentData<'a>) -> Self {
        Self { info, data }
    }

    /// Create new typed component data from its parts
    #[inline]
    pub fn from_parts(
        info: ComponentInfoPtr,
        data: OwnedPtr<'a>,
        changed_at: Tick,
        added_at: Tick,
    ) -> Self {
        Self {
            info,
            data: UntypedComponentData::new(data, changed_at, added_at),
        }
    }

    /// Drops the component's data.
    #[inline]
    pub fn drop(self) {
        self.info.drop(self.data.data)
    }
}

/// Holds information about a removed entity from an [`Archetype`]
pub(crate) struct RemovedEntity<'a> {
    /// If the removed entity was swapped with another, this is the id of the swapped entity
    /// (now located at the location of the removed entity), **None** if no swap occured (i.e.
    /// it was the last entity)
    pub swapped: Option<EntityId>,
    /// Components data of the removed entity
    pub components: Vec<TypedComponentData<'a>>,
}

impl<'a> RemovedEntity<'a> {
    /// Create new removed entity info
    #[inline]
    pub fn new(swapped: Option<EntityId>, capacity: usize) -> Self {
        Self {
            swapped,
            components: Vec::with_capacity(capacity),
        }
    }
}

/// Unique identifier for an archetype, based on hash of its component types.
/// Received from [`Archetype::hash_types`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeId(u64);

#[derive(Debug)]
pub struct Archetype {
    /// Unique id of this archetype computed from its types
    id: ArchetypeId,
    /// Stores component type ids and their index in `self.components`
    /// TODO: use ComponentLocation here ?
    types: Vec<(TypeId, usize, ComponentInfoPtr)>,
    /// Entity ids in this archetype, `self.entity_ids[entity]` has components at
    /// `self.components[N][entity]` `
    entity_ids: Vec<EntityId>,
    /// Component buckets
    pub components: Vec<ComponentsData>,
}

impl Archetype {
    /// Create new archetype with `types`.
    pub fn new(id: ArchetypeId, mut infos: Vec<ComponentInfoPtr>) -> Self {
        Self::sort_infos(&mut infos);

        let components = infos.iter().map(|t| ComponentsData::new(*t)).collect();

        let types = infos
            .into_iter()
            .enumerate()
            .map(|(i, v)| (v.as_ref().type_id, i, v))
            .collect::<Vec<_>>();

        assert!(
            !types.windows(2).any(|w| w[0].0 == w[1].0),
            "Duplicate types in archetype"
        );

        Self {
            id,
            entity_ids: Vec::new(),
            types,
            components,
        }
    }

    /// Insert new entity with components matching this archetype, returns its location
    ///
    /// # Safety
    /// Caller must ensure that `components` match this archetype's types, both in type and amount.
    /// Components will be inserted in order so they must be **sorted by type id**.
    #[must_use]
    pub(super) unsafe fn insert_entity(
        &mut self,
        entity_id: EntityId,
        components: Vec<TypedComponentData>,
    ) -> EntityLocation {
        debug_assert!(
            self.types.len() == components.len()
                && components
                    .iter()
                    .zip(self.types.iter())
                    .all(|(comp, (type_id, ..))| comp.info.as_ref().type_id == *type_id),
            "Component types mismatch with archetype types"
        );

        self.entity_ids.push(entity_id);

        for (i, component) in components.into_iter().enumerate() {
            self.components[i].insert(component.data);
        }

        debug_assert!(
            self.components
                .iter()
                .all(|row| row.len() == self.entity_ids.len()),
            "Specific components length mismatch with entity IDs length"
        );
        debug_assert!(
            self.components.len() == self.types.len(),
            "Components length mismatch with types length"
        );

        EntityLocation::new(self.id, self.len() - 1)
    }

    /// Remove entity, returns removed entity data
    ///
    /// # Panics
    /// Panics if entity is not valid in this archetype
    #[must_use]
    pub(super) fn remove_entity(
        &mut self,
        entity_id: EntityId,
        entity_location: EntityLocation,
    ) -> RemovedEntity<'_> {
        self.validate_entity(entity_id, entity_location);
        let entity_index = entity_location.index();

        self.entity_ids.swap_remove(entity_index);
        let swapped = self.entity_ids.get(entity_index).copied();
        let mut removed = RemovedEntity::new(swapped, self.components.len());

        for (i, components_data) in self.components.iter_mut().enumerate() {
            let info_ptr = self.types[i].2;
            let removed_data = components_data.remove(entity_index);

            removed
                .components
                .push(TypedComponentData::new(info_ptr, removed_data))
        }

        removed
    }

    /// Sets component to a new value
    ///
    /// # Panics
    /// Panics if entity is not valid in this archetype or if component type does not exist in this
    /// archetype
    pub(super) fn set_component(
        &mut self,
        entity_id: EntityId,
        location: EntityLocation,
        component: TypedComponentData,
    ) {
        self.validate_entity(entity_id, location);

        let type_id = component.info.as_ref().type_id;
        let component_index = self.component_index(&type_id);
        let index = location.index();

        if index >= self.len() {
            component.drop();
            panic!(
                "Entity index {} out of bounds for archetype with {} entities",
                index,
                self.len()
            );
        }

        self.components[component_index].set(index, component.data);
    }

    /// Debug assert that entity is valid in this archetype
    #[inline(always)]
    fn validate_entity(&self, entity_id: EntityId, location: EntityLocation) {
        debug_assert!(
            self.id == location.archetype_id(),
            "Entity {:?} location archetype id {:?} does not match current archetype id {:?}",
            entity_id,
            location.archetype_id(),
            self.id
        );

        debug_assert!(
            location.index() < self.len(),
            "Entity {:?} location index {} out of bounds for archetype with {} entities",
            entity_id,
            location.index(),
            self.len()
        );

        debug_assert!(
            self.entity_ids[location.index()] == entity_id,
            "Entity {:?} not found at its location index {} in archetype",
            entity_id,
            location.index()
        );
    }

    /// Amount of entities in this archetype
    #[inline]
    pub fn len(&self) -> usize {
        self.entity_ids.len()
    }

    /// Returns true if there are no entities in this archetype
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entity_ids.is_empty()
    }

    /// Returns archetype id
    #[inline]
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    /// Returns a pointer to the [`ComponentsData`] at `index`
    #[inline]
    pub(crate) fn get_components_data_mut(&mut self, index: usize) -> *mut ComponentsData {
        &mut self.components[index]
    }

    /// Returns sorted infos
    #[inline]
    fn sort_infos(types: &mut [ComponentInfoPtr]) {
        types.sort_by_key(|info| info.as_ref().type_id);
    }

    /// Returns sorted component infos of this archetype
    #[inline]
    pub fn infos(&self) -> Vec<ComponentInfoPtr> {
        self.types.iter().map(|(_, _, v)| *v).collect()
    }

    /// Check if `type_id` exists in self
    #[inline]
    pub fn has_type(&self, type_id: &TypeId) -> bool {
        self.try_component_index(type_id).is_some()
    }

    /// Check if all [`QueryComponentType::Normal`] types exist in self
    fn has_query_types(&self, type_ids: &[QueryComponentType]) -> bool {
        type_ids.iter().all(|type_id| match type_id {
            QueryComponentType::Normal(type_id) => self.has_type(type_id),
            QueryComponentType::Option(_) => true,
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

    /// Check if any of `type_ids` exist in self
    #[inline]
    pub fn has_types_any(&self, type_ids: &[TypeId]) -> bool {
        type_ids.iter().any(|type_id| self.has_type(type_id))
    }

    /// Check if archetype has `entity_id`. It is slower than checking by location, becuause it
    /// uses a linear search.
    #[inline]
    pub fn has_entity_by_id(&self, entity_id: &EntityId) -> bool {
        self.entity_ids.contains(entity_id)
    }

    /// Check if archetype has `entity_id` at `location`
    #[inline]
    pub fn has_entity(&self, entity_id: &EntityId, location: &EntityLocation) -> bool {
        self.id == location.archetype_id()
            && self.entity_ids.get(location.index()) == Some(entity_id)
    }

    /// Get component index in `types` if it exists
    ///
    /// # Panics
    /// Panics if component type does not exist in this archetype
    #[inline]
    pub fn component_index(&self, component_id: &TypeId) -> usize {
        self.try_component_index(component_id)
            .expect("Component type not found in archetype")
    }

    /// Get component index in `types` if it exists
    #[inline]
    pub fn try_component_index(&self, component_id: &TypeId) -> Option<usize> {
        self.types
            .iter()
            .find_map(|(id, i, _)| if id == component_id { Some(*i) } else { None })
    }

    /// Returns hash of sorted types as [`ArchetypeId`]
    ///
    /// # Safety
    /// Caller must ensure that `components` contains no duplicates, is not epmty, and is **sorted
    /// by component type id** or the resulting hash will be invalid.
    pub(super) unsafe fn hash_sorted_components(
        components: &mut [TypedComponentData],
    ) -> ArchetypeId {
        debug_assert!(!components.is_empty(), "Cannot hash empty component types");
        debug_assert!(
            components
                .windows(2)
                .all(|w| w[0].info.as_ref().type_id < w[1].info.as_ref().type_id),
            "Archetype types are not sorted by type id"
        );

        let mut hasher = DefaultHasher::new();
        for component in components {
            let type_id = component.info.as_ref().type_id;
            type_id.hash(&mut hasher);
        }

        let hash = hasher.finish();
        ArchetypeId(hash)
    }
}

pub(crate) struct TickFilterIndices {
    changed: Vec<Vec<usize>>,
    added: Vec<Vec<usize>>,
}

impl TickFilterIndices {
    #[inline]
    fn changed_empty(&self) -> bool {
        self.changed.len() == 1 && self.changed[0].is_empty()
    }

    #[inline]
    fn added_empty(&self) -> bool {
        self.added.len() == 1 && self.added[0].is_empty()
    }
}

impl Archetype {
    /// Returns archetypes with matching [`query types`](QueryComponentType) and filters, and component indices for
    /// `changed` filters acquired from [`Archetype::get_changed_filter_indices`]
    pub(crate) fn filtered(
        &mut self,
        type_ids: &[QueryComponentType],
        filters: &mut Filters,
    ) -> Option<TickFilterIndices> {
        if self.has_query_types(type_ids) && self.passes_type_filters(filters) {
            // Safety: we have already checked that all changed filters exist in archetype
            Some(self.get_changed_filter_indices(filters))
        } else {
            None
        }
    }

    /// Evaluates type filters against this archetype.
    /// Does **NOT** check `tick` filters, only their type existence in the archetype.
    fn passes_type_filters(&self, filters: &mut Filters) -> bool {
        if filters.empty {
            return true;
        }

        // Changed
        self.has_types(&filters.changed)
            // Added
            && self.has_types(&filters.added)
            // With
            && self.has_types(&filters.with)
            // Without
            && filters
                .without
                .iter()
                .all(|type_id| !self.has_type(type_id))
            // Or
            && filters
                .or
                .iter_mut()
                .all(|filters| self.passes_type_filters_any(filters))
    }

    /// Returns true if any of the type filters evaluate to true
    fn passes_type_filters_any(&self, filters: &mut Filters) -> bool {
        assert!(filters.or.is_empty(), "Nested OR filters are not supported");

        if filters.empty {
            return true;
        }

        // Any existence filters
        filters.matches_existence = self.has_types_any(&filters.with)
            || filters
                .without
                .iter()
                .any(|type_id| !self.has_type(type_id));

        // For matching we include existence of tick filters, but we do not store it,
        // because further `tick` checks are required, so we can't skip them later.
        filters.matches_existence
            || self.has_types_any(&filters.changed)
            || self.has_types_any(&filters.added)
    }

    /// Returns indices of requested `tick` fields in this archetype, where first vec is from
    /// `filters.tick_based` and the rest (optional) are from `filters.or[n].tick_based`.
    ///
    /// # Note
    /// Tick filters are either `Changed<T>` or `Added<T>`.
    ///
    /// # Panics
    /// Panics if type_id in `filters.tick_based` is not found in archetype
    fn get_changed_filter_indices(&self, filters: &Filters) -> TickFilterIndices {
        let mut changed = Vec::with_capacity(1);
        let mut added = Vec::with_capacity(1);

        let changed_base = filters
            .changed
            .iter()
            .map(|component_id| self.component_index(component_id))
            .collect::<Vec<_>>();
        changed.push(changed_base);

        let added_base = filters
            .added
            .iter()
            .map(|component_id| self.component_index(component_id))
            .collect::<Vec<_>>();
        added.push(added_base);

        for or_filters in &filters.or {
            // Existence filters already matched, so we can skip tick based `Or` checks
            if or_filters.matches_existence {
                continue;
            }

            let changed_or_indices = or_filters
                .changed
                .iter()
                .filter_map(|component_id| self.try_component_index(component_id))
                .collect::<Vec<_>>();

            let added_or_indices = or_filters
                .added
                .iter()
                .filter_map(|component_id| self.try_component_index(component_id))
                .collect::<Vec<_>>();

            // Tick filters validation
            if changed_or_indices.is_empty() && added_or_indices.is_empty() {
                if or_filters.with.len() + or_filters.without.len() == 0 {
                    panic!("Or<T> filter only contains `tick_based` filters, but none of the types are found in archetype");
                } else {
                    panic!("Or<T> filter doesn't match existence filters, and none of the Changed<T> | Added<T> types are found in archetype");
                }
            }

            changed.push(changed_or_indices);
            added.push(added_or_indices);
        }

        TickFilterIndices { changed, added }
    }

    /// Checks if requested fields (indices) are marked as changed in entities[at]
    ///
    /// # Note
    /// To get the correct indices call `archetype.get_changed_filter_indices(filters)`
    pub(crate) fn check_changed_fields(
        &self,
        at: usize,
        indices: &TickFilterIndices,
        system_last_run: Tick,
    ) -> bool {
        // Changed<T> base filter
        let changed_base = if indices.changed_empty() {
            true
        } else {
            indices.changed[0]
                .iter()
                .all(|&index| self.components[index].changed_since(at, system_last_run))
        };

        if !changed_base {
            // short circuit
            return false;
        }

        // Added<T> base filter
        let added_base = if indices.added_empty() {
            true
        } else {
            indices.added[0]
                .iter()
                .all(|&index| self.components[index].added_since(at, system_last_run))
        };

        if !added_base {
            // short circuit
            return false;
        }

        // Or<T> filters
        let changed_or = indices.changed.iter().skip(1);
        let added_or = indices.added.iter().skip(1);
        let or = changed_or
            .zip(added_or)
            .all(|(changed_or_indices, added_or_indices)| {
                let changed = changed_or_indices
                    .iter()
                    .any(|&index| self.components[index].changed_since(at, system_last_run));
                if changed {
                    // short circuit
                    return true;
                }

                let added = added_or_indices
                    .iter()
                    .any(|&index| self.components[index].added_since(at, system_last_run));
                changed || added
            });

        or
    }
}
