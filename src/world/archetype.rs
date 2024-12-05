use std::{any::{Any, TypeId}, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use super::entities::EntityId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ArchetypeId(u64);

#[derive(Debug)]
pub(crate) struct Archetype {
    pub(super) entity_ids: Vec<EntityId>,
    pub(super) types: HashMap<TypeId, usize>,

    /// Per entity components are currently stored together in a row
    /// ```
    /// vec![
    ///  vec![A, B, C], -> entity 1
    ///  vec![A, B, C], -> entity 2
    ///  vec![A, B, C], -> entity 3
    /// ]
    /// ```
    pub components: Vec<Vec<Box<dyn Any>>>,
}

impl Archetype {
    pub fn new(types: Vec<TypeId>) -> Self {
        let original_len = types.len();
        let types = Self::sort_types(types);
        let types = types.into_iter().enumerate()
            .map(|(i, v)| (v, i))
            .collect::<HashMap<TypeId, usize>>();

        assert!(types.len() == original_len, "Duplicate types in archetype");

        let components = types.iter().map(|_| Vec::new()).collect();

        Self {
            entity_ids: Vec::new(),
            types,
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
            let index = self.types[&type_id];
            self.components[index].push(component);
        }

        assert!(
            self.components.iter().all(|row| row.len() == self.entity_ids.len()), 
            "Specific component length mismatch with entity IDs length"
        );
        assert!(
            self.components.len() == self.types.len(),
            "Components length mismatch with types length"
        );
    }

    /// Remove entity, returns removed components or None if entity_id doesn't exist
    pub(super) fn remove_entity(&mut self, entity_id: EntityId) -> Option<Vec<Box<dyn Any>>> {
        if let Some(index) = self.entity_ids.iter().position(|id| *id == entity_id) {
            self.entity_ids.remove(index);

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
        if let Some(index) = self.entity_ids.iter().position(|id| *id == entity_id) {
            let type_id = (*component).type_id();
            let component_index = self.types[&type_id];
            self.components[component_index][index] = component;
            return true
        }
        false
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
    pub fn has_type(&self, type_id: TypeId) -> bool {
        self.types.contains_key(&type_id)
    }

    /// Check if all type_ids exist in self
    pub fn has_types(&self, type_ids: &[TypeId]) -> bool {
        type_ids.iter().all(|type_id| self.has_type(*type_id))
    }

    /// Check if all type_ids exist in self, no more no less
    pub fn has_types_all(&self, type_ids: &[TypeId]) -> bool {
        self.types.len() == type_ids.len() && self.has_types(type_ids)
    }

    /// Same as has_type but with generic T type
    pub fn has_t<T: 'static>(self) -> bool {
        self.has_type(TypeId::of::<T>())
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
