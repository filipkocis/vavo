use std::{any::{Any, TypeId}, collections::HashMap, ops::{Deref, DerefMut}};

pub struct Resources {
    resources: HashMap<TypeId, Box<dyn Any>>,
}

/// Immutable resource reference
pub struct Res<T>(pub(crate) *const T);

/// Mutable resource reference
pub struct ResMut<T>(pub(crate) *mut T);

impl<T> Deref for Res<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> Deref for ResMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T> DerefMut for ResMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl Resources {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource)); 
    }

    pub fn get<T: 'static>(&self) -> Option<Res<T>> {
        self.resources.get(&TypeId::of::<T>()).map(|r| Res(r.downcast_ref::<T>().unwrap()))
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<ResMut<T>> {
        self.resources.get_mut(&TypeId::of::<T>()).map(|r| ResMut(r.downcast_mut::<T>().unwrap()))
    }
}
