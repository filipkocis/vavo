use std::{any::TypeId, collections::HashMap, ops::Deref};

use crate::{assets::{Assets, Handle}, system::SystemsContext, world::EntityId};

use super::RenderHandle;

pub trait RenderAsset<R> {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        entity_id: Option<&EntityId>
    ) -> R;
}

/// Wrapper for render asset entry to allow multiple mutable borrows for RenderAssets<T>
pub struct RenderAssetEntry<T>(*const T);

impl<T> Clone for RenderAssetEntry<T> {
    fn clone(&self) -> Self {
        RenderAssetEntry(self.0)
    }
}

impl<T> Deref for RenderAssetEntry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }   
    }
}

/// Generic handle for Asset of any type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AssetHandleId(TypeId, u64);

/// ID combining entity id and component type id
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EntityComponentId(u32, TypeId);

impl<T: 'static> Into<AssetHandleId> for &Handle<T> {
    fn into(self) -> AssetHandleId {
        AssetHandleId(TypeId::of::<T>(), self.id())
    }
}

impl<T: 'static> Into<EntityComponentId> for (&EntityId, &T) {
    fn into(self) -> EntityComponentId {
        EntityComponentId(self.0.raw(), TypeId::of::<T>())
    }
}

pub struct RenderAssets<T> {
    storage: HashMap<RenderHandle<T>, T>,
    handle_map: HashMap<AssetHandleId, RenderHandle<T>>,
    entity_component_map: HashMap<EntityComponentId, RenderHandle<T>>,
    next_id: u64,
}

impl<T> RenderAssets<T> {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            handle_map: HashMap::new(),
            entity_component_map: HashMap::new(),
            next_id: 0,
        }
    }

    fn step_id(&mut self) -> RenderHandle<T> {
        let id = self.next_id;
        self.next_id += 1;
        RenderHandle::new(id)
    }

    pub fn insert(&mut self, asset: T) -> RenderHandle<T> {
        let id = self.step_id();
        self.storage.insert(id.clone(), asset);
        id
    }

    pub fn get(&self, handle: &RenderHandle<T>) -> Option<&T> {
        self.storage.get(&handle)
    }

    pub fn get_by_entity<A>(
        &mut self, 
        entity_id: &EntityId, 
        component: &A, 
        ctx: &mut SystemsContext,
    ) -> RenderAssetEntry<T>
    where A: 'static + RenderAsset<T> {
        let entity_component_id = (entity_id, component).into();

        let rae = match self.entity_component_map.get(&entity_component_id){
            Some(key) => {
                self.storage
                    .entry(key.clone())
                    .or_insert_with(|| component.create_render_asset(ctx, Some(entity_id)))
            },
            None => {
                let key = self.insert(component.create_render_asset(ctx, Some(entity_id)));
                self.entity_component_map.insert(entity_component_id, key.clone());
                self.storage.get(&key).unwrap()
            }
        };

        RenderAssetEntry(rae)
    }

    pub fn get_by_handle<A>(
        &mut self, 
        handle: &Handle<A>, 
        ctx: &mut SystemsContext,
    ) -> RenderAssetEntry<T>
    where A: 'static + RenderAsset<T> {
        let rae = match self.handle_map.get(&handle.into()){
            Some(key) => {
                self.storage
                    .entry(key.clone())
                    .or_insert_with(|| Self::create_asset(handle, ctx))
            },
            None => {
                let key = self.insert(Self::create_asset(handle, ctx));
                self.handle_map.insert(handle.into(), key.clone());
                self.storage.get(&key).unwrap()
            }
        };

        RenderAssetEntry(rae)
    }

    fn create_asset<A>(handle: &Handle<A>, ctx: &mut SystemsContext) -> T
    where A: 'static + RenderAsset<T> {
        let assets = ctx.resources.get::<Assets<A>>().expect("Assets<A> not found");
        let asset = assets.get(handle).expect("Asset not found, invalid handle");
        let render_asset = asset.create_render_asset(ctx, None);

        render_asset
    }

    pub fn remove<A: 'static>(&mut self, handle: &Handle<A>) -> Option<T> {
        // TODO: should we remove both the handle and the asset? 
        let key = self.handle_map.remove(&handle.into())?;
        self.storage.remove(&key)
    }
}
