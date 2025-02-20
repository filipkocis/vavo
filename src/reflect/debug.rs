use std::fmt::{Debug};

use super::{type_info::{ArrayInfo, EnumInfo, PrimitiveInfo, StructInfo, TupleInfo, TypeInfo}, Reflect};

impl Debug for dyn Reflect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug_fmt(!f.alternate())) 
    }
}

impl dyn Reflect {
    fn internal_debug_fmt(&self, inline: bool, indent: usize) -> String {
        let type_info = self.type_info();
        
        match type_info {
            TypeInfo::Primitive(info) => write_primitive(self, info),
            TypeInfo::Struct(info) => write_struct(self, info, inline, indent),
            TypeInfo::Enum(info) => write_enum(self, info, inline, indent),
            TypeInfo::Array(info) => write_array(self, info, inline, indent),
            TypeInfo::Tuple(info) => write_tuple(self, info, inline, indent),

            _ => todo!()
        }
    }

    pub fn debug_fmt(&self, inline: bool) -> String {
        self.internal_debug_fmt(inline, 0)
    }
}

fn write_array(value: &dyn Reflect, info: ArrayInfo, inline: bool, indent: usize) -> String {
    let mut s = String::new();

    if info.is_list {
        s.push_str(info.path.name);
    }

    s.push('[');
    if !inline { s.push('\n'); }

    let mut i = 0;
    loop {
        let Some(field) = value.field_by_index(i) else {
            if !inline { s.push('\n'); }
            break;
        };
        if i != 0 {
            s.push(',');
            if !inline { s.push('\n'); }
        }
        
        s.push_str(&indent_str(inline, indent + 1, i != 0));
        s.push_str(&field.internal_debug_fmt(inline, indent + 1));     
        i += 1;
    }

    if i == 0 {
        while s.pop() != Some('[') {}
        s.push_str("[]");
    } else {
        s.push_str(&indent_str(inline, indent, false));
        s.push(']');
    }

    s
}

fn write_tuple(value: &dyn Reflect, info: TupleInfo, inline: bool, indent: usize) -> String {
    let range = 0..info.element_types.len();
    let range_end = info.element_types.len() - 1;
    let mut s = String::new();

    s.push('(');
    if !inline { s.push('\n'); }

    for i in range {
        let field = value.field_by_index(i).expect("field_by_index failed, incorrect element_types");
        s.push_str(&indent_str(inline, indent + 1, i != 0));
        s.push_str(&field.internal_debug_fmt(inline, indent + 1));
        if i != range_end {
            s.push(',');
        }
        if !inline { s.push('\n'); }
    }

    s.push_str(&indent_str(inline, indent, false));
    s.push(')');

    s
}

fn write_enum(value: &dyn Reflect, info: EnumInfo, inline: bool, indent: usize) -> String {
    let mut s = String::from(info.path.path);
    s.push_str("::");

    s.push('(');
    if !inline { s.push('\n'); }

    // TODO: handle different enum variants, and print variant name
    let mut i = 0;
    loop {
        let Some(field) = value.field_by_index(i) else {
            if !inline { s.push('\n'); }
            break;
        };
        if i != 0 {
            s.push(',');
            if !inline { s.push('\n'); }
        }
        
        s.push_str(&indent_str(inline, indent + 1, i != 0));
        s.push_str(&field.internal_debug_fmt(inline, indent + 1));     
        i += 1;
    }

    if i == 0 {
        while s.pop() != Some('(') {}
    } else {
        s.push_str(&indent_str(inline, indent, false));
        s.push(')');
    }

    s
}

/// Return indentation string. 4 spaces per indent level.
fn indent_str(inline: bool, indent: usize, use_space: bool) -> String {
    match (inline, indent, use_space) {
        (true, 0, _) => return String::new(),
        (true, _, true) => return " ".to_string(),
        (true, _, _) => return String::new(),
        _ => {}
    }

    " ".repeat(indent * 4)
}

fn write_struct(value: &dyn Reflect, info: StructInfo, inline: bool, indent: usize) -> String {
    let range = 0..info.field_names.len();
    let range_end = info.field_names.len() - 1;
    let mut s = String::from(info.path.path);

    if info.is_tuple {
        s.push('('); 
        if !inline { s.push('\n'); }

        for i in range {
            let field = value.field_by_index(i).expect("field_by_index failed, incorrect field_names");
            s.push_str(&indent_str(inline, indent + 1, i != 0));
            s.push_str(&field.internal_debug_fmt(inline, indent + 1));
            if i != range_end {
                s.push(',');
            }
            if !inline { s.push('\n'); }
        }

        s.push_str(&indent_str(inline, indent, false));
        s.push(')');
    } else {
        s.push_str(" {");
        if !inline { s.push('\n'); }

        for i in range {
            let field = value.field_by_index(i).expect("field_by_index failed, incorrect field_names");
            s.push_str(&indent_str(inline, indent + 1, true));
            s.push_str(&info.field_names[i]);
            s.push_str(": ");
            s.push_str(&field.internal_debug_fmt(inline, indent + 1));
            if i != range_end {
                s.push(',');
            }
            if !inline { s.push('\n'); }
        }

        s.push_str(&indent_str(inline, indent, true));
        s.push('}');
    }

    s
}

macro_rules! gen_write_primitive {
    ($($type:ident),+) => {
        fn write_primitive(value: &dyn Reflect, info: PrimitiveInfo) -> String {
            let path = info.path;

            match path.name {
                // types which can't be used as an macro arg
                "&'static str" => {
                    match value.downcast_ref::<&str>() {
                        Some(value) => format!("{:?}", value),
                        None => panic!("info path {:?} doesn't match value type {:?}", path.name, "str")
                    }
                },

                // macro expansion
                $(
                    stringify!($type) => {
                        match value.downcast_ref::<$type>() {
                            Some(value) => format!("{:?}", value),
                            None => panic!("info path {:?} doesn't match value type {:?}", path.name, stringify!($type))
                        }
                    },
                )+
                name => {
                    format!("{{{:?}}}", name)
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
