use std::{any::type_name, collections::VecDeque};

use super::{TupleInfo, ArrayInfo, EnumInfo, GetTypeInfo, PrimitiveInfo, TypeInfo, TypePathInfo};

/// Implement GetTypeInfo for primitive types separated with commas.
macro_rules! impl_primitive {
    ($($type:ty),+) => {$(
        impl GetTypeInfo for $type {
            fn type_info(&self) -> TypeInfo {
                TypeInfo::Primitive(PrimitiveInfo::new(TypePathInfo::new(
                    stringify!($type),
                    type_name::<Self>()
                )))
            }

            fn type_name(&self) -> &'static str {
                stringify!($type)
            }
        }
    )+}
}

impl_primitive!(
    u8, u16, u32, u64, u128, usize, 
    i8, i16, i32, i64, i128, isize, 
    f32, f64, bool, char, str, String
);

/// Implement GetTypeInfo for enum types separated with commas.
///
/// Foo = Bar Baz : "optional module path",
/// Foo = : "optional module path",
/// Foo = Bar Baz,
/// Foo = ,
macro_rules! impl_enum {
    ($($type:ty = $($variant:ident)* $(: $desc:literal)?),+) => {$(
        impl GetTypeInfo for $type {
            fn type_info(&self) -> TypeInfo {
                TypeInfo::Enum(EnumInfo::new(TypePathInfo::new(
                    stringify!($type),
                    type_name::<Self>()
                ), [$(stringify!($variant)),*]))
            }

            fn type_name(&self) -> &'static str {
                stringify!($type)
            }
        }
    )+}
}

impl<T: GetTypeInfo> GetTypeInfo for Option<T> {
    fn type_info(&self) -> TypeInfo {
        TypeInfo::Enum(EnumInfo::new(
            TypePathInfo::new("Option", type_name::<Self>()),
            ["None", "Some"],
        ))
    }

    fn type_name(&self) -> &'static str {
        "Option"
    }
}

impl<T: GetTypeInfo, E: GetTypeInfo> GetTypeInfo for Result<T, E> {
    fn type_info(&self) -> TypeInfo {
        TypeInfo::Enum(EnumInfo::new(
            TypePathInfo::new("Result", type_name::<Self>()),
            ["Err", "Ok"],
        ))
    }

    fn type_name(&self) -> &'static str {
        "Result"
    }
}

/// Implement GetTypeInfo for list types separated with commas.
macro_rules! impl_list {
    ($($type:ident<$($generic:ident),+>),+) => {$(
        impl<$($generic: GetTypeInfo),+> GetTypeInfo for $type<$($generic),+> {
            fn type_info(&self) -> TypeInfo {
                TypeInfo::Array(ArrayInfo::new(
                    TypePathInfo::new(
                        stringify!($type),
                        type_name::<Self>()
                    ), 
                    self.get(0).map(|v| v.type_info()),
                    true,
                    0
                ))
            }

            fn type_name(&self) -> &'static str {
                stringify!($type)
            }
        }
    )+}
}

impl_list!(Vec<T>, VecDeque<T>);

impl<T: GetTypeInfo, const N: usize> GetTypeInfo for [T; N] {
    fn type_info(&self) -> TypeInfo {
        TypeInfo::Array(ArrayInfo::new(
            TypePathInfo::new(
                stringify!([T; N]),
                type_name::<Self>()
            ),
            self.get(0).map(|v| v.type_info()),
            false,
            N
        ))
    }

    fn type_name(&self) -> &'static str {
        stringify!([T; N])
    }
}

/// Implement GetTypeInfo for tuple types separated with commas.
macro_rules! impl_tuple {
    ($(($($type:ident),+) ($($index:tt),+)),+) => {$(
        impl<$($type: GetTypeInfo),+> GetTypeInfo for ($($type),+) {
            fn type_info(&self) -> TypeInfo {
                TypeInfo::Tuple(TupleInfo::new(TypePathInfo::new(
                    stringify!(($($type),+)),
                    type_name::<Self>()
                ), [$(self.$index.type_info()),+]))
            }

            fn type_name(&self) -> &'static str {
                stringify!(($($type),+))
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
