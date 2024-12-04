use std::{any::TypeId, collections::HashMap};

use crate::{assets::{Assets, Handle}, prelude::Resources};

use super::RenderHandle;

pub trait RenderAsset<R> {
    fn create_render_asset(&self, device: &wgpu::Device, resources: &mut Resources) -> R;
}

/// Generic handle for Asset of any type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AssetHandleId(TypeId, u64);

impl<T: 'static> Into<AssetHandleId> for &Handle<T> {
    fn into(self) -> AssetHandleId {
        AssetHandleId(TypeId::of::<T>(), self.id())
    }
}

pub struct RenderAssets<T> {
    storage: HashMap<RenderHandle<T>, T>,
    handle_map: HashMap<AssetHandleId, RenderHandle<T>>,
    next_id: u64,
}

impl<T> RenderAssets<T> {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            handle_map: HashMap::new(),
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

    pub fn get_or_create<A>(
        &mut self, 
        handle: &Handle<A>, 
        device: &wgpu::Device, 
        resources: &mut Resources
    ) -> &T
    where A: 'static + RenderAsset<T> {
        match self.handle_map.get(&handle.into()){
            Some(key) => {
                self.storage
                    .entry(key.clone())
                    .or_insert_with(|| Self::create_asset(handle, device, resources))
            },
            None => {
                let key = self.insert(Self::create_asset(handle, device, resources));
                self.handle_map.insert(handle.into(), key.clone());
                self.storage.get(&key).unwrap()
            }
        }
    }

    fn create_asset<A>(handle: &Handle<A>, device: &wgpu::Device, resources: &mut Resources) -> T
    where A: 'static + RenderAsset<T> {
        let assets = resources.get::<Assets<A>>().unwrap();
        let asset = assets.get(handle).unwrap();
        let render_asset = asset.create_render_asset(device, resources);

        render_asset
    }

    pub fn remove<A: 'static>(&mut self, handle: &Handle<A>) -> Option<T> {
        // TODO: should we remove both the handle and the asset? 
        let key = self.handle_map.remove(&handle.into())?;
        self.storage.remove(&key)
    }
}
