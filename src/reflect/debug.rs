use std::fmt::{Debug, Formatter};

use super::{type_info::{PrimitiveInfo, StructInfo, TypeInfo}, Reflect};

impl Debug for dyn Reflect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_info = self.type_info();
        
        match type_info {
            TypeInfo::Primitive(info) => write_primitive(f, info, self),
            TypeInfo::Struct(info) => write_struct(f, info, self),

            _ => todo!()
        }
    }
}

fn write_struct(f: &mut Formatter<'_>, info: StructInfo, value: &dyn Reflect) -> std::fmt::Result {
    let name = &info.path.name;
    let range = 0..info.field_names.len();

    if info.is_tuple {
        let mut f = f.debug_tuple(name);
        
        range.for_each(|i| {
            let field = value.field_by_index(i).expect("field_by_index not found, invalid field_names");
            let field = format!("{:?}", field);
            f.field(&field);
        });

        f.finish()
    } else {
        let mut f = f.debug_struct(&info.path.name);

        range.for_each(|i| {
            let field = value.field_by_index(i).expect("field_by_index not found, invalid field_names");
            let field = format!("{:?}", field);
            f.field(&info.field_names[i], &field);
        });

        f.finish()
    }
}

macro_rules! gen_write_primitive {
    ($($type:ident),+) => {
        fn write_primitive(f: &mut Formatter<'_>, info: PrimitiveInfo, value: &dyn Reflect) -> std::fmt::Result {
            let path = info.path;

            match path.name {
                // types which can't be used as an macro arg
                "&'static str" => {
                    match value.downcast_ref::<&str>() {
                        Some(value) => write!(f, "{:?}", value),
                        None => panic!("info path {:?} doesn't match value type {:?}", path.name, "str")
                    }
                },

                // macro expansion
                $(
                    stringify!($type) => {
                        match value.downcast_ref::<$type>() {
                            Some(value) => write!(f, "{:?}", value),
                            None => panic!("info path {:?} doesn't match value type {:?}", path.name, stringify!($type))
                        }
                    },
                )+
                name => {
                    write!(f, "{{{:?}}}", name)
                }
            }
        }
    };
}

gen_write_primitive!(
    u8, u16, u32, u64, u128, usize, 
    i8, i16, i32, i64, i128, isize, 
    f32, f64, bool, char, String
);
