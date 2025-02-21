use std::{any::{Any, TypeId}, collections::{HashMap, HashSet, VecDeque}};

use type_info::GetTypeInfo;

pub mod type_info;
pub mod inspector;
mod debug;

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

impl dyn Reflect {
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        let t = TypeId::of::<T>();
        let concrete = self.type_id();

        t == concrete
    }

    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn Reflect as *const T)) }
        } else {
            None
        }
    }

    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn Reflect as *mut T)) }
        } else {
            None
        }
    }
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
                self.get(index).map(|value| value as &dyn Reflect)
            }

            fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
                if self.len() <= index {
                    return Err(value);
                }

                value.downcast::<$($generic)?>().map(|v| self[index] = *v)
            }
        }
    )+}
}

impl_list!(Vec<T>, VecDeque<T>);

impl<T: Reflect, const N: usize> Reflect for [T; N] {
    fn field_by_index(&self, index: usize) -> Option<&dyn Reflect> {
        self.get(index).map(|value| value as &dyn Reflect)
    }

    fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        if self.len() <= index {
            return Err(value);
        }

        value.downcast::<T>().map(|v| self[index] = *v) 
    }
}

/// Implement GetTypeInfo for tuple types separated with commas.
macro_rules! impl_tuple {
    ($(($($type:ident),+) ($($index:tt),+)),+) => {$(
        impl<$($type: Reflect),+> Reflect for ($($type),+) {
            fn field_by_index(&self, index: usize) -> Option<&dyn Reflect> {
                match index {
                    $($index => Some(&self.$index as &dyn Reflect),)+
                    _ => None
                }
            }

            fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
                match index {
                    $($index => value.downcast::<_>().map(|v| self.$index = *v),)+
                    _ => Err(value)
                }
            }
        }
    )+}
}

impl_tuple!(
    (T1, T2) (0, 1),
    (T1, T2, T3) (0, 1, 2),
    (T1, T2, T3, T4) (0, 1, 2, 3),
    (T1, T2, T3, T4, T5) (0, 1, 2, 3, 4),
    (T1, T2, T3, T4, T5, T6) (0, 1, 2, 3, 4, 5),
    (T1, T2, T3, T4, T5, T6, T7) (0, 1, 2, 3, 4, 5, 6),
    (T1, T2, T3, T4, T5, T6, T7, T8) (0, 1, 2, 3, 4, 5, 6, 7)
);

impl<T: Reflect, U: Reflect> Reflect for HashMap<T, U> {
    fn field_by_index(&self, _: usize) -> Option<&dyn Reflect> {
        None
    }

    fn set_field_by_index(&mut self, _: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        Err(value)
    }
}

impl<T: Reflect> Reflect for HashSet<T> {
    fn field_by_index(&self, _: usize) -> Option<&dyn Reflect> {
        None
    }

    fn set_field_by_index(&mut self, _: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
        Err(value)
    }
}

/// Implement Reflection for struct types
// TODO: make this a proc macro (all of these macros should be)
macro_rules! impl_struct {
    ($($type:ident $is_tuple:tt ($($field_index:tt $field:tt),*) $(: ($($generic:ident),+))? ),+) => {$(
        impl$(<$($generic: Reflect),+>)? Reflect for $type$(<$($generic),+>)? {
            fn field_by_index(&self, index: usize) -> Option<&dyn Reflect> {
                match index {
                    $($field_index => Some(&self.$field),)*
                    _ => None
                }
            }

            fn set_field_by_index(&mut self, index: usize, value: Box<dyn Any>) -> Result<(), Box<dyn Any>> {
                match index {
                    $($field_index => value.downcast::<_>().map(|v| self.$field = *v),)*
                    _ => Err(value)
                }
            }
        }
    )+}
}

mod glam_impls {
    use super::*;
    use glam::*;

    impl_struct!(
        UVec2 false (0 x, 1 y), UVec3 false (0 x, 1 y, 2 z), UVec4 false (0 x, 1 y, 2 z, 3 w),
        Vec2 false (0 x, 1 y), Vec3 false (0 x, 1 y, 2 z), Vec4 false (0 x, 1 y, 2 z, 3 w),
        Mat2 false (0 x_axis, 1 y_axis), Mat3 false (0 x_axis, 1 y_axis, 2 z_axis), Mat4 false (0 x_axis, 1 y_axis, 2 z_axis, 3 w_axis),
        Quat false (0 x, 1 y, 2 z, 3 w)
    );
}
