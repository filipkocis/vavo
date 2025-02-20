use std::{any::Any, collections::VecDeque};

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

                value.downcast::<$type>().map(|value| *self = *value)
            }
        }
    )+}
}

impl_primitive!(
    u8, u16, u32, u64, u128, usize, 
    i8, i16, i32, i64, i128, isize, 
    f32, f64, bool, char
);

impl Reflect for str {
    fn field_by_index(&self, _: usize) -> Option<&dyn Reflect> {
        None
    }

    fn set_field_by_index(&mut self, _: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        Err(value)
    }
}

impl Reflect for &'static str {
    fn field_by_index(&self, _: usize) -> Option<&dyn Reflect> {
        None
    }

    fn set_field_by_index(&mut self, _: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        Err(value)
    }
}

impl Reflect for String {
    fn field_by_index(&self, _: usize) -> Option<&dyn Reflect> {
        None
    }

    fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        if index != 0 {
            return Err(value);
        }

        value.downcast::<String>().map(|value| *self = *value)
    }
}

impl<T: Reflect> Reflect for Option<T> {
    fn field_by_index(&self, index: usize) -> Option<&dyn Reflect> {
        match self {
            Some(value) if index == 0 => Some(value),
            _ => None,
        }
    }

    fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        match self {
            Some(v) if index == 0 => value.downcast::<T>().map(|value| *v = *value),
            _ => Err(value),
        }
    }
}

impl<T: Reflect, E: Reflect> Reflect for Result<T, E> {
    fn field_by_index(&self, index: usize) -> Option<&dyn Reflect> {
        match self {
            Ok(value) if index == 0 => Some(value),
            Err(value) if index == 0 => Some(value),
            _ => None,
        }
    }

    fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        match self {
            Ok(v) if index == 0 => value.downcast::<T>().map(|value| *v = *value),
            Err(e) if index == 0 => value.downcast::<E>().map(|value| *e = *value),
            _ => Err(value),
        }
    }
}

/// Implement Reflection for list types
macro_rules! impl_list {
    ($($type:ident<$($generic:ident),+>),+) => {$(
        impl<$($generic: Reflect),+> Reflect for $type<$($generic),+> {
            fn field_by_index(&self, index: usize) -> Option<&dyn Reflect> {
                self.get(index)
                    .map(|value| value as &dyn Reflect)
            }

            fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
                if self.len() <= index {
                    return Err(value);
                }

                value.downcast::<$($generic)?>()
                    .map(|v| self[index] = *v)
            }
        }
    )+}
}

impl_list!(Vec<T>, VecDeque<T>);
