use std::{any::{Any, TypeId}, collections::HashMap, ops::{Deref, DerefMut}};

use crate::{assets::{AssetLoader, Assets, ShaderLoader}, render_assets::{BindGroup, Buffer, Pipeline, RenderAssets}, renderer::{Image, Material, Mesh, Texture}};

use super::{FixedTime, Resource, Time};

pub struct Resources {
    resources: HashMap<TypeId, Box<dyn Any>>,
}

/// Immutable resource reference. 
/// Holds a raw pointer to the resource.
pub struct Res<T>(pub(crate) *const T);

/// Mutable resource reference.
/// Holds a raw mutable pointer to the resource.
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

    /// Used by [Commands](crate::system::Commands) to insert resources
    pub(crate) fn insert_boxed(&mut self, type_id: TypeId, boxed_resource: Box<dyn Any>) {
        self.resources.insert(type_id, boxed_resource); 
    }

    pub fn insert<R: Resource>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource)); 
    }

    pub fn remove(&mut self, type_id: TypeId) {
        self.resources.remove(&type_id);
    }

    pub fn get<R: Resource>(&self) -> Option<Res<R>> {
        self.resources.get(&TypeId::of::<R>()).map(|r| Res(r.downcast_ref::<R>().unwrap()))
    }

    pub fn get_mut<R: Resource>(&mut self) -> Option<ResMut<R>> {
        self.resources.get_mut(&TypeId::of::<R>()).map(|r| ResMut(r.downcast_mut::<R>().unwrap()))
    }

    /// Initialize self with default resources
    pub(crate) fn insert_default_resources(&mut self) {
        // assets
        self.insert(Assets::<Mesh>::new());
        self.insert(Assets::<Material>::new());
        self.insert(Assets::<Image>::new());

        // render assets
        self.insert(RenderAssets::<Buffer>::new());
        self.insert(RenderAssets::<BindGroup>::new());
        self.insert(RenderAssets::<Pipeline>::new());
        self.insert(RenderAssets::<Texture>::new());

        // resources
        self.insert(AssetLoader::new());
        self.insert(ShaderLoader::new());
    }

    /// Update some builtin resources
    pub(crate) fn update(&mut self) {
        self.get_mut::<Time>().unwrap().update();
        self.get_mut::<FixedTime>().unwrap().update();
    }
}
