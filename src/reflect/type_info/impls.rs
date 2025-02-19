use std::{any::type_name, collections::{HashMap, HashSet, VecDeque}};

use super::{ArrayInfo, EnumInfo, GetTypeInfo, MapInfo, PrimitiveInfo, SetInfo, StructInfo, TupleInfo, TypeInfo, TypePathInfo};

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

impl<T: GetTypeInfo, U: GetTypeInfo> GetTypeInfo for HashMap<T, U> {
    fn type_info(&self) -> TypeInfo {
        TypeInfo::Map(MapInfo::new(
            TypePathInfo::new(
                "HashMap",
                type_name::<Self>()
            ),
            self.keys().next().map(|v| v.type_info()),
            self.values().next().map(|v| v.type_info()),
        ))
    }

    fn type_name(&self) -> &'static str {
        "HashMap"
    }
}

impl<T: GetTypeInfo> GetTypeInfo for HashSet<T> {
    fn type_info(&self) -> TypeInfo {
        TypeInfo::Set(SetInfo::new(
            TypePathInfo::new(
                "HashSet",
                type_name::<Self>()
            ),
            self.into_iter().next().map(|v| v.type_info()),
        ))
    }

    fn type_name(&self) -> &'static str {
        "HashSet"
    }
}

/// Implement GetTypeInfo for struct types separated with commas.
///
/// Foo is_tuple (fields) : (generics),
///
/// Foo false (a),
/// Foo false (a) : (T),
/// Foo true (0, 1),
macro_rules! impl_struct {
    ($($type:ident $is_tuple:tt ($($field:tt),+) $(: ($($generic:ident),+))? ),+) => {$(
        impl$(<$($generic: GetTypeInfo),+>)? GetTypeInfo for $type$(<$($generic),+>)? {
            fn type_info(&self) -> TypeInfo {
                TypeInfo::Struct(StructInfo::new(
                    TypePathInfo::new(
                        stringify!($type),
                        type_name::<Self>()
                    ), 
                    [$(stringify!($field)),+],
                    [$(self.$field.type_info()),+],
                    $is_tuple,
                ))
            }

            fn type_name(&self) -> &'static str {
                stringify!($type)
            }
        }
    )+}
}

mod glam_impls {
    use super::*;
    use glam::*;

    impl_struct!(
        UVec2 false (x, y), UVec3 false (x, y, z), UVec4 false (x, y, z, w),
        Vec2 false (x, y), Vec3 false (x, y, z), Vec4 false (x, y, z, w),
        Mat2 false (x_axis, y_axis), Mat3 false (x_axis, y_axis, z_axis), Mat4 false (x_axis, y_axis, z_axis, w_axis)
    );
}
