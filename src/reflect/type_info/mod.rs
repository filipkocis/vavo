mod impls;

/// Trait which provides information about a concrete type at runtime
pub trait GetTypeInfo {
    fn type_info(&self) -> TypeInfo;
    fn type_name(&self) -> &'static str;
}

/// Information about a concrete type implementing the [`Reflect`](super::Reflect) trait.
#[derive(Debug, Clone)]
pub enum TypeInfo {
    Primitive(PrimitiveInfo),
    Struct(StructInfo),
    Enum(EnumInfo),
    Array(ArrayInfo),
    Tuple(TupleInfo),
    Map(MapInfo),
    Set(SetInfo),
}

impl TypeInfo {
    pub fn info(&self) -> &TypePathInfo {
        match self {
            Self::Primitive(info) => &info.path,
            Self::Struct(info) => &info.path,
            Self::Enum(info) => &info.path,
            Self::Array(info) => &info.path,
            Self::Tuple(info) => &info.path,
            Self::Map(info) => &info.path,
            Self::Set(info) => &info.path,
        }
    }

    pub fn name(&self) -> &'static str {
        self.info().name
    }

    pub fn path(&self) -> &'static str {
        self.info().path
    }

    pub fn field_names(&self) -> Option<&[&'static str]> {
        match self {
            Self::Struct(info) => Some(&info.field_names),
            _ => None,
        }
    }

    pub fn field_types(&self) -> Option<&[TypeInfo]> {
        match self {
            Self::Struct(info) => Some(&info.field_types),
            Self::Tuple(info) => Some(&info.element_types),
            _ => None,
        }
    }

    pub fn field_name_by_index(&self, index: usize) -> Option<&'static str> {
        match self {
            Self::Struct(info) => info.field_names.get(index).copied(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypePathInfo {
    pub name: &'static str,
    pub path: &'static str,
}

impl TypePathInfo {
    pub fn new(name: &'static str, path: &'static str) -> Self {
        Self { name, path }
    }
}

#[derive(Debug, Clone)]
pub struct PrimitiveInfo {
    pub path: TypePathInfo,
}

impl PrimitiveInfo {
    pub fn new(path: TypePathInfo) -> Self {
        Self { path }
    }
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub path: TypePathInfo,
    pub field_names: Box<[&'static str]>,
    pub field_types: Box<[TypeInfo]>,
    pub is_tuple: bool,
}

impl StructInfo {
    pub fn new<const N: usize>(path: TypePathInfo, field_names: [&'static str; N], field_types: [TypeInfo; N], is_tuple: bool) -> Self {
        Self {
            path,
            field_names: field_names.into(),
            field_types: field_types.into(),
            is_tuple,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub path: TypePathInfo,
    pub variant_names: Box<[&'static str]>,
    // pub variant_types: Box<[TypeInfo]>,
}

impl EnumInfo {
    pub fn new<const N: usize>(path: TypePathInfo, variant_names: [&'static str; N]) -> Self {
        Self {
            path,
            variant_names: variant_names.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayInfo {
    pub path: TypePathInfo,
    pub element_type: Option<Box<TypeInfo>>,
    pub is_list: bool,
    pub length: usize,
}

impl ArrayInfo {
    pub fn new(path: TypePathInfo, element_type: Option<TypeInfo>, is_list: bool, length: usize) -> Self {
        Self {
            path,
            element_type: element_type.map(Box::new),
            is_list,
            length,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TupleInfo {
    pub path: TypePathInfo,
    pub element_types: Box<[TypeInfo]>,
}

impl TupleInfo {
    pub fn new<const N: usize>(path: TypePathInfo, element_types: [TypeInfo; N]) -> Self {
        Self {
            path,
            element_types: element_types.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MapInfo {
    pub path: TypePathInfo,
    pub key_type: Option<Box<TypeInfo>>,
    pub value_type: Option<Box<TypeInfo>>,
}

impl MapInfo {
    pub fn new(path: TypePathInfo, key_type: Option<TypeInfo>, value_type: Option<TypeInfo>) -> Self {
        Self {
            path,
            key_type: key_type.map(Box::new),
            value_type: value_type.map(Box::new),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetInfo {
    pub path: TypePathInfo,
    pub element_type: Option<Box<TypeInfo>>,
}

impl SetInfo {
    pub fn new(path: TypePathInfo, element_type: Option<TypeInfo>) -> Self {
        Self {
            path,
            element_type: element_type.map(Box::new),
        }
    }
}
