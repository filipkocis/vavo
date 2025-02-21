use std::{any::{Any, TypeId}, collections::HashMap};

use super::Reflect;

/// Function which transforms a value into a [`Reflect`] trait object.
pub type ReflectTransformer = fn(&dyn Any) -> &dyn Reflect;

/// Type Registry for reflectable types. It is used to transform unknown components into
/// [`Reflect`] trait objects.
///
/// Use [`App::register_type`](crate::app::App) to register new types.
pub struct ReflectTypeRegistry {
    type_ids: HashMap<TypeId, ReflectTransformer>,
}

impl ReflectTypeRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self {
            type_ids: HashMap::new(),
        }
    }

    /// Register new reflectable type.
    pub fn register<T: Reflect>(&mut self) {
        self.type_ids.insert(TypeId::of::<T>(), |value| {
            value.downcast_ref::<T>().unwrap()
        });
    }

    /// Returns the [`ReflectTransformer`] for the given type id.
    pub fn get(&self, type_id: TypeId) -> Option<ReflectTransformer> {
        self.type_ids.get(&type_id).copied()
    }

    /// Reflects the given value if it is registered.
    pub fn reflect<'a>(&self, value: &'a dyn Any) -> Option<&'a dyn Reflect> {
        let type_id = value.type_id();
        self.get(type_id).map(|transformer| transformer(value))
    }
}
