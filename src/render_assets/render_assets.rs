use std::{any::TypeId, collections::HashMap};

use crate::{assets::{Assets, Handle}, prelude::Resources};

pub trait RenderAsset<R> {
    fn create_render_asset(&self, device: &wgpu::Device, resources: &mut Resources) -> R;
}

pub struct RenderAssets<T> {
    storage: HashMap<(TypeId, u64), T>,
}

impl<T> RenderAssets<T> {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    // fn step_id(&mut self) -> Handle<T> {
    //     let id = self.next_id;
    //     self.next_id += 1;
    //     Handle::new(id)
    // }
    //
    // pub fn add(&mut self, asset: T) -> Handle<T> {
    //     let id = self.step_id();
    //     self.storage.insert(id.clone(), asset);
    //     id
    // }

    fn get_key<A: 'static>(handle: &Handle<A>) -> (TypeId, u64) {
        (TypeId::of::<A>(), handle.id())
    }

    pub fn get<A>(&mut self, handle: &Handle<A>, device: &wgpu::Device, resources: &mut Resources) -> &T
    where A: 'static + RenderAsset<T> {
        self.storage.entry(Self::get_key(handle)).or_insert_with(|| Self::create_asset(handle, device, resources))
    }

    fn create_asset<A>(handle: &Handle<A>, device: &wgpu::Device, resources: &mut Resources) -> T
    where A: 'static + RenderAsset<T> {
        let assets = resources.get::<Assets<A>>().unwrap();
        let asset = assets.get(handle).unwrap();
        let render_asset = asset.create_render_asset(device, resources);

        render_asset
    }

    pub fn remove<A: 'static>(&mut self, handle: &Handle<A>) -> Option<T> {
        self.storage.remove(&Self::get_key(handle))
    }
}
