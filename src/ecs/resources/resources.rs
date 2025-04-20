use std::{
    any::TypeId,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{
    assets::{AssetLoader, Assets, ShaderLoader},
    ecs::{
        ptr::{DataPtr, DataPtrMut},
        resources::{FixedTime, Resource, Time},
        store::blob::BlobVec,
        tick::{Tick, TickStamp, TickStampMut},
    },
    render_assets::{BindGroup, Buffer, Pipeline, RenderAssets},
    renderer::{Image, Material, Mesh, Texture},
};

/// Holds a type-erased resource and its metadata.
pub(crate) struct ResourceData {
    type_id: TypeId,
    data: BlobVec,
    changed_at: Tick,
    added_at: Tick,
}

impl ResourceData {
    /// Creates a new resource data instance.
    pub(crate) fn new<R: Resource>(resource: R, current_tick: Tick) -> Self {
        let type_id = TypeId::of::<R>();
        let mut data = BlobVec::new_type::<R>(1);
        // Safety: type and value is correct
        unsafe {
            data.push(resource);
        }

        Self {
            type_id,
            data,
            changed_at: current_tick,
            added_at: current_tick,
        }
    }

    #[inline]
    /// Sets tick metadata to `current_tick`, useful when you don't have access to the
    /// `current_tick` during resource creation.
    pub(crate) fn set_tick(&mut self, current_tick: Tick) {
        self.changed_at = current_tick;
        self.added_at = current_tick;
    }

    #[inline]
    /// Returns immutable [`TickStamp`] for the resource.
    fn get_ticks(&self, current_tick: Tick) -> TickStamp {
        TickStamp::new(&self.changed_at, &self.added_at, current_tick)
    }

    #[inline]
    /// Returns mutable [`TickStampMut`] for the resource.
    fn get_ticks_mut(&mut self, current_tick: Tick) -> TickStampMut {
        TickStampMut::new(&mut self.changed_at, &mut self.added_at, current_tick)
    }
}

/// Immutable resource reference.
/// Holds a raw pointer to the resource.
pub struct Res<R: Resource>(pub(crate) DataPtr<R>);

impl<R: Resource> Deref for Res<R> {
    type Target = R;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// Mutable resource reference.
/// Holds a raw mutable pointer to the resource.
pub struct ResMut<R: Resource>(pub(crate) DataPtrMut<R>);

impl<R: Resource> Deref for ResMut<R> {
    type Target = R;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<R: Resource> DerefMut for ResMut<R> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

/// Storage for all resources in a world.
pub struct Resources {
    resources: HashMap<TypeId, ResourceData>,
    current_tick: *const Tick,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            current_tick: std::ptr::null(),
        }
    }

    /// Initialize tick pointer, necessary for correct tick tracking.
    /// Done during [world](crate::prelude::World) initialization.
    pub fn initialize_tick(&mut self, current_tick: *const Tick) {
        self.current_tick = current_tick
    }

    #[inline]
    /// Returns the current world tick.
    fn tick(&self) -> Tick {
        unsafe { *self.current_tick }
    }

    /// Used by [Commands](crate::system::Commands) to insert resources
    pub(crate) fn insert_resource_data(&mut self, type_id: TypeId, resource_data: ResourceData) {
        self.resources.insert(type_id, resource_data);
    }

    /// Insert new resource into the world.
    pub fn insert<R: Resource>(&mut self, resource: R) {
        self.resources
            .insert(TypeId::of::<R>(), ResourceData::new(resource, self.tick()));
    }

    /// Remove a resource from the world.
    pub fn remove(&mut self, type_id: TypeId) {
        self.resources.remove(&type_id);
    }

    /// Get a resource by type.
    pub fn get<R: Resource>(&self) -> Option<Res<R>> {
        self.resources.get(&TypeId::of::<R>()).map(|r| {
            Res(DataPtr::new(
                // Safety: type is correct and index is always valid
                unsafe { r.data.get(0) },
                r.get_ticks(self.tick()),
            ))
        })
    }

    /// Get a mutable resource by type.
    pub fn get_mut<R: Resource>(&mut self) -> Option<ResMut<R>> {
        let current_tick = self.tick();
        self.resources.get_mut(&TypeId::of::<R>()).map(|r| {
            ResMut(DataPtrMut::new(
                // Safety: type is correct and index is always valid
                unsafe { r.data.get(0) },
                r.get_ticks_mut(current_tick),
            ))
        })
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
