use std::any::TypeId;

/// A type which can be used as an entity component in the ECS. 
pub trait Component: Send + Sync + 'static {
    fn get_type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}
