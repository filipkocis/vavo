use std::any::Any;

use type_info::GetTypeInfo;

pub mod type_info;

/// Trait enabling dynamic reflection of types. Any type implementing this trait can be
/// inspected and mutated at runtime.
pub trait Reflect: GetTypeInfo + Any + Send + Sync + 'static {
    fn field_names(&self) -> Vec<&'static str> {
        self.type_info().field_names().unwrap_or_default().to_vec()
    }
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        let index = self.field_names().iter().position(|n| n == &name)?;
        self.field_by_index(index)
    }
    fn field_by_index(&self, index: usize) -> Option<&dyn Reflect>;

    fn set_field(&mut self, name: &str, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        let index = match self.field_names().iter().position(|n| n == &name) {
            Some(index) => index,
            None => return Err(value),
        };
        self.set_field_by_index(index, value)
    }
    fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>>;
}

/// Implement reflection for primitive types.
macro_rules! impl_primitive {
    ($($type:ident),+) => {$(
        impl Reflect for $type {
            fn field_by_index(&self, index: usize) -> Option<&dyn Reflect> {
                if index != 0 {
                    return None
                }
                Some(self)
            }

            fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
                if index != 0 {
                    return Err(value);
                }

                if let Some(value) = value.downcast_ref::<$type>() {
                    *self = *value;
                    Ok(())
                } else {
                    Err(value)
                }
            }
        }
    )+}
}

impl_primitive!(
    u8, u16, u32, u64, u128, usize, 
    i8, i16, i32, i64, i128, isize, 
    f32, f64, bool, char // str, String
);
