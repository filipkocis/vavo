use std::{
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use crate::{
    assets::{AssetLoader, Assets, ShaderLoader},
    ecs::{
        ptr::{DataPtr, DataPtrMut, OwnedPtr},
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
    #[inline]
    /// Creates a new resource data instance.
    pub(crate) fn new<R: Resource>(resource: R, current_tick: Tick) -> Self {
        let type_id = TypeId::of::<R>();
        let mut data = BlobVec::new_type::<R>(1);
        unsafe {
            let mut resource = ManuallyDrop::new(resource);
            // Safety: resource is pushed and not used afterwards.
            let ptr = OwnedPtr::new_ref(&mut resource);
            // Safety: type and value are correct
            data.push(ptr);
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
    fn get_ticks(&self, current_tick: Tick, last_run: Tick) -> TickStamp {
        TickStamp::new(&self.changed_at, &self.added_at, current_tick, last_run)
    }

    #[inline]
    /// Returns mutable [`TickStampMut`] for the resource.
    fn get_ticks_mut(&mut self, current_tick: Tick, last_run: Tick) -> TickStampMut {
        TickStampMut::new(
            &mut self.changed_at,
            &mut self.added_at,
            current_tick,
            last_run,
        )
    }
}

#[repr(transparent)]
/// Immutable resource reference.
/// Holds a raw pointer to the resource.
pub struct Res<R: Resource>(pub(crate) DataPtr, PhantomData<R>);

impl<R: Resource> Deref for Res<R> {
    type Target = R;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.raw().cast::<R>() }
    }
}

#[repr(transparent)]
/// Mutable resource reference.
/// Holds a raw mutable pointer to the resource.
pub struct ResMut<R: Resource>(pub(crate) DataPtrMut, PhantomData<R>);

impl<R: Resource> Deref for ResMut<R> {
    type Target = R;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.raw().cast::<R>() }
    }
}

impl<R: Resource> DerefMut for ResMut<R> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.mark_changed();
        // We just marked it as changed
        self.deref_mut_no_change()
    }
}

/// Storage for all resources in a world.
#[derive(Default)]
pub struct Resources {
    resources: HashMap<TypeId, ResourceData>,
    current_tick: *const Tick,
    system_last_run: Tick,
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize tick pointer, necessary for correct tick tracking.
    /// Done during [world](crate::prelude::World) initialization.
    pub fn initialize_tick(&mut self, current_tick: *const Tick) {
        self.current_tick = current_tick
    }

    /// Returns the current world tick.
    #[inline]
    fn tick(&self) -> Tick {
        debug_assert!(
            !self.current_tick.is_null(),
            "Resources tick pointer is null. Did you forget to call `initialize_tick`?",
        );
        unsafe { *self.current_tick }
    }

    /// Sets the `last_run` tick
    #[inline]
    pub(crate) fn set_system_last_run(&mut self, last_run: Tick) {
        self.system_last_run = last_run;
    }

    /// Used by [Commands](crate::system::Commands) to insert resources
    pub(crate) fn insert_resource_data(&mut self, type_id: TypeId, resource_data: ResourceData) {
        self.resources.insert(type_id, resource_data);
    }

    /// Check if a resource of type R exists in the world.
    #[inline]
    pub fn contains<R: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
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

    /// Get a resource by type, or `None` if it doesn't exist.
    pub fn try_get<R: Resource>(&self) -> Option<Res<R>> {
        self.resources.get(&TypeId::of::<R>()).map(|r| {
            let data = DataPtr::new(
                // Safety: type is correct and index is always valid
                unsafe { r.data.get(0) },
                r.get_ticks(self.tick(), self.system_last_run),
            );
            Res(data, PhantomData)
        })
    }

    /// Get a mutable resource by type, or `None` if it doesn't exist.
    pub fn try_get_mut<R: Resource>(&mut self) -> Option<ResMut<R>> {
        let current_tick = self.tick();
        self.resources.get_mut(&TypeId::of::<R>()).map(|r| {
            let data = DataPtrMut::new(
                // Safety: type is correct and index is always valid
                unsafe { r.data.get(0) },
                r.get_ticks_mut(current_tick, self.system_last_run),
            );
            ResMut(data, PhantomData)
        })
    }

    /// Get a resource by type. **Panics** if the resource doesn't exist.
    #[inline]
    pub fn get<R: Resource>(&self) -> Res<R> {
        match self.try_get::<R>() {
            Some(res) => res,
            None => panic!(
                "Cannot get resource {:?} because it does not exist",
                std::any::type_name::<R>()
            ),
        }
    }

    /// Get a mutable resource by type. **Panics** if the resource doesn't exist.
    #[inline]
    pub fn get_mut<R: Resource>(&mut self) -> ResMut<R> {
        match self.try_get_mut::<R>() {
            Some(res) => res,
            None => panic!(
                "Cannot get mutable resource {:?} because it does not exist",
                std::any::type_name::<R>()
            ),
        }
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
        self.get_mut::<Time>().update();
        self.get_mut::<FixedTime>().update();
    }
}
