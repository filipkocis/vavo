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
    pub key_type: Box<TypeInfo>,
    pub value_type: Box<TypeInfo>,
}

#[derive(Debug, Clone)]
pub struct SetInfo {
    pub path: TypePathInfo,
    pub element_type: Box<TypeInfo>,
}
