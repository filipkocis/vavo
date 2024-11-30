use std::{
    any::{Any, TypeId}, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}, ops::{Add, Sub}
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId(u32);
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

// has to be public for now because of query.rs
#[derive(Debug)]
pub struct Archetype {
    entity_ids: Vec<EntityId>,
    types: HashMap<TypeId, usize>,
    pub components: Vec<Vec<Box<dyn Any>>>,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeId(u64);

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
    fn insert_entity(&mut self, entity_id: EntityId, components: Vec<Box<dyn Any>>) {
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
    fn remove_entity(&mut self, entity_id: EntityId) -> Option<Vec<Box<dyn Any>>> {
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
    fn update_component(&mut self, entity_id: EntityId, component: Box<dyn Any>) -> bool {
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
    pub fn hash_types(types: Vec<TypeId>) -> ArchetypeId {
        let mut hasher = DefaultHasher::new();
        let types = Self::sort_types(types);

        for type_id in types {
            type_id.hash(&mut hasher);
        }

        let hash = hasher.finish();
        ArchetypeId(hash)
    } 
}

#[derive(Debug)]
pub struct Entities {
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

    /// Exposes archetypes
    pub fn archetypes(&mut self) -> &mut HashMap<ArchetypeId, Archetype> {
        &mut self.archetypes
    }

    /// Exposes next entity ID
    pub fn next_entity_id(&self) -> EntityId {
        self.next_entity_id
    }

    /// Step next entity ID counter
    /// Returns new entity ID
    pub fn step_entity_id(&mut self) -> EntityId {
        self.next_entity_id = self.next_entity_id + 1;
        self.next_entity_id
    }

    /// Spawn new entity with components
    pub fn spawn_entity(&mut self, entity_id: EntityId, components: Vec<Box<dyn Any>>) {
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
    pub fn despawn_entity(&mut self, entity_id: EntityId) {
        if let Some(archetype) = self.archetypes.values_mut().find(|a| a.entity_ids.contains(&entity_id)) {
            archetype.remove_entity(entity_id);
        }
    }

    /// Insert new component
    pub fn insert_component(&mut self, entity_id: EntityId, component: Box<dyn Any>) {
        if let Some(archetype) = self.archetypes.values_mut().find(|a| a.entity_ids.contains(&entity_id)) {
            let type_id = (*component).type_id(); 
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
    pub fn remove_component(&mut self, entity_id: EntityId, type_id: TypeId) {
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
