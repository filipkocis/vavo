use std::{any::TypeId, collections::HashMap, ops::Deref, sync::Arc};

use crate::{assets::{Asset, Assets, Handle}, prelude::Res, ecs::resources::Resource, system::SystemsContext, ecs::world::EntityId};

use super::{RenderAsset, RenderHandle};

pub trait IntoRenderAsset<R: RenderAsset> {
    fn create_render_asset(
        &self, 
        ctx: &mut SystemsContext,
        entity_id: Option<&EntityId>
    ) -> R;
}

/// Wrapper for render asset entry to allow multiple mutable borrows for RenderAssets<T>
pub struct RenderAssetEntry<T>(Arc<T>);

impl<T> Clone for RenderAssetEntry<T> {
    fn clone(&self) -> Self {
        RenderAssetEntry(self.0.clone())
    }
}

impl<T> Deref for RenderAssetEntry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// ID for a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ResourceId(TypeId);

/// Generic handle for Asset of any type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AssetHandleId(TypeId, u64);

/// ID combining entity id and component type id
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EntityComponentId(u32, TypeId);

impl<T: 'static> Into<ResourceId> for &Res<T> {
    fn into(self) -> ResourceId {
        ResourceId(TypeId::of::<T>())
    }
}

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

#[derive(crate::macros::Resource)]
pub struct RenderAssets<RA: RenderAsset> {
    storage: HashMap<RenderHandle<RA>, Arc<RA>>,
    handle_map: HashMap<AssetHandleId, RenderHandle<RA>>,
    entity_component_map: HashMap<EntityComponentId, RenderHandle<RA>>,
    resource_map: HashMap<ResourceId, RenderHandle<RA>>,
    next_id: u64,
}

impl<RA: RenderAsset> RenderAssets<RA> {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            handle_map: HashMap::new(),
            entity_component_map: HashMap::new(),
            resource_map: HashMap::new(),
            next_id: 0,
        }
    }

    fn step_id(&mut self) -> RenderHandle<RA> {
        let id = self.next_id;
        self.next_id += 1;
        RenderHandle::new(id)
    }

    pub fn insert(&mut self, asset: RA) -> RenderHandle<RA> {
        let id = self.step_id();
        self.storage.insert(id.clone(), Arc::new(asset));
        id
    }

    pub fn get(&self, handle: &RenderHandle<RA>) -> Option<Arc<RA>> {
        self.storage.get(&handle).cloned()
    }

    pub fn get_by_entity<A>(
        &mut self, 
        entity_id: &EntityId, 
        component: &A, 
        ctx: &mut SystemsContext,
    ) -> RenderAssetEntry<RA>
    where A: IntoRenderAsset<RA> + 'static {
        let entity_component_id = (entity_id, component).into();

        let rae = match self.entity_component_map.get(&entity_component_id){
            Some(key) => {
                self.storage
                    .entry(key.clone())
                    .or_insert_with(|| Arc::new(component.create_render_asset(ctx, Some(entity_id))))
            },
            None => {
                let key = self.insert(component.create_render_asset(ctx, Some(entity_id)));
                self.entity_component_map.insert(entity_component_id, key.clone());
                self.storage.get(&key).unwrap()
            }
        };

        RenderAssetEntry(rae.clone())
    }

    pub fn get_by_handle<A>(
        &mut self, 
        handle: &Handle<A>, 
        ctx: &mut SystemsContext,
    ) -> RenderAssetEntry<RA>
    where A: Asset + IntoRenderAsset<RA> {
        let asset_handle_id = handle.into();

        let rae = match self.handle_map.get(&asset_handle_id){
            Some(key) => {
                self.storage
                    .entry(key.clone())
                    .or_insert_with(|| Arc::new(Self::create_asset(handle, ctx)))
            },
            None => {
                let key = self.insert(Self::create_asset(handle, ctx));
                self.handle_map.insert(asset_handle_id, key.clone());
                self.storage.get(&key).unwrap()
            }
        };

        RenderAssetEntry(rae.clone())
    }

    pub fn get_by_resource<R>(
        &mut self,
        resource: &Res<R>,
        ctx: &mut SystemsContext,
        replace: bool,
    ) -> RenderAssetEntry<RA>
    where R: Resource + IntoRenderAsset<RA> {
        let resource_id = resource.into();

        let rae = match self.resource_map.get(&resource_id){
            Some(key) => {
                if replace {
                    self.storage.remove(key);
                }

                self.storage
                    .entry(key.clone())
                    .or_insert_with(|| Arc::new(resource.create_render_asset(ctx, None)))
            },
            None => {
                let key = self.insert(resource.create_render_asset(ctx, None));
                self.resource_map.insert(resource_id, key.clone());
                self.storage.get(&key).unwrap()
            }
        };

        RenderAssetEntry(rae.clone())
    }

    fn create_asset<A>(handle: &Handle<A>, ctx: &mut SystemsContext) -> RA
    where A: Asset + IntoRenderAsset<RA> {
        let assets = ctx.resources.get::<Assets<A>>().expect("Assets<A> not found");
        let asset = assets.get(handle).expect("Asset not found, invalid handle");
        let render_asset = asset.create_render_asset(ctx, None);

        render_asset
    }

    pub fn remove<A: 'static>(&mut self, handle: &Handle<A>) -> Option<Arc<RA>> {
        // TODO: should we remove both the handle and the asset? 
        let key = self.handle_map.remove(&handle.into())?;
        self.storage.remove(&key)
    }
    
    /// Remove render asset created by `get_by_entity` method
    pub fn remove_by_entity<A: 'static>(&mut self, entity_id: &EntityId, component: &A) -> Option<Arc<RA>> {
        let entity_component_id = (entity_id, component).into();
        let key = self.entity_component_map.remove(&entity_component_id)?;
        self.storage.remove(&key)
    }
}
