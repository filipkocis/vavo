use std::any::Any;

use type_info::GetTypeInfo;

pub mod type_info;

/// Trait enabling dynamic reflection of types. Any type implementing this trait can be
/// inspected and mutated at runtime.
pub trait Reflect: GetTypeInfo + Any + Send + Sync + 'static {
    fn field_names(&self) -> Vec<&'static str>;
    fn field(&self, name: &str) -> Option<&dyn Reflect>;
    fn field_by_index(&self, index: usize) -> Option<&dyn Reflect>;

    fn set_field(&mut self, name: &str, value: Box<dyn Any>);
    fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>);
}
