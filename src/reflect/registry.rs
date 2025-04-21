use std::{any::TypeId, collections::HashMap};

use crate::ecs::ptr::UntypedPtrLt;

use super::Reflect;

/// Function which transforms a value into a [`Reflect`] trait object.
pub type ReflectTransformer = for<'a> fn(UntypedPtrLt<'a>) -> &'a dyn Reflect;

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
        self.type_ids.insert(TypeId::of::<T>(), |value| unsafe {
            value.as_ptr().cast::<T>().as_ref()
        });
    }

    /// Returns the [`ReflectTransformer`] for the given type id.
    pub fn get(&self, type_id: TypeId) -> Option<ReflectTransformer> {
        self.type_ids.get(&type_id).copied()
    }

    /// Reflects the given value if it is registered.
    pub fn reflect<'a>(&self, value: UntypedPtrLt<'a>, type_id: TypeId) -> Option<&'a dyn Reflect> {
        self.get(type_id).map(|transformer| transformer(value))
    }
}

impl Default for ReflectTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
