use super::{EnumInfo, GetTypeInfo, PrimitiveInfo, TypeInfo, TypePathInfo};

/// Implement GetTypeInfo for primitive types separated with commas.
macro_rules! impl_primitive {
    ($($type:ty),+) => {$(
        impl GetTypeInfo for $type {
            fn type_info(&self) -> TypeInfo {
                TypeInfo::Primitive(PrimitiveInfo::new(TypePathInfo::new(
                    stringify!($type),
                    concat!("std::", stringify!($type)),
                    "std",
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
    f32, f64, bool, char, str, 
    String, ()
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
                    concat!("std::", 
                        $(concat!(stringify!($desc), "::"), )?
                        stringify!($type)
                    ),
                    concat!("std", $("::", stringify!($desc))?)
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
            TypePathInfo::new("Option", "std::option::Option", "std::option"),
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
            TypePathInfo::new("Result", "std::result::Result", "std::result"),
            ["Err", "Ok"],
        ))
    }

    fn type_name(&self) -> &'static str {
        "Result"
    }
}
