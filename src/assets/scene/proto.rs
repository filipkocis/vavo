use std::sync::Arc;

use crate::{
    assets::Scene,
    prelude::{Component, EntityId, World},
};

type BuildFn<T> = dyn Fn(&mut World, EntityId) -> T + Send + Sync + 'static;
type WithWorldFn<T> = dyn Fn(&mut T, &mut World, EntityId) + Send + Sync + 'static;

/// Post-build modifier for a component `T` built by a [Proto], can modify the component in
/// multiple ways.
#[derive(Clone)]
enum Modifier<T> {
    With(Arc<dyn Fn(&mut T) + Send + Sync + 'static>),
    WithTake(Arc<dyn Fn(T) -> T + Send + Sync + 'static>),
    WithWorld(Arc<WithWorldFn<T>>),
}

impl<T> Modifier<T> {
    /// Call all `modifiers` on `component` and return the final component
    #[inline]
    fn call(modifiers: &[Modifier<T>], mut component: T, world: &mut World, entity: EntityId) -> T {
        for modifier in modifiers {
            match modifier {
                Modifier::With(f) => f(&mut component),
                Modifier::WithTake(f) => component = f(component),
                Modifier::WithWorld(f) => f(&mut component, world, entity),
            }
        }
        component
    }
}

#[derive(Clone)]
/// Prototype for building a component of type `T` in a scene, can have shared state between
/// builds.
///
/// Allows mapping and modification of the built component before insertion into the ECS world. All
/// modifiers are applied in the order they were added.
///
/// Can be used to create `base` prototypes since `Proto` is [Clone] via internal [Arc] usage.
pub struct Proto<T> {
    /// Function to build the component `T`
    build_fn: Arc<BuildFn<T>>,
    /// Modifiers to apply to the component `T` after building
    modifiers: Vec<Modifier<T>>,
}

impl<T> Proto<T> {
    /// Create a new Proto component with the given build function
    #[inline]
    #[must_use]
    pub fn new<F>(build_fn: F) -> Self
    where
        F: Fn(&mut World, EntityId) -> T + Send + Sync + 'static,
    {
        Self {
            build_fn: Arc::new(Box::new(build_fn)),
            modifiers: Vec::new(),
        }
    }

    /// Build the component `T` using the prototype's build function
    #[inline]
    #[must_use]
    pub fn build_component(&self, world: &mut World, entity: EntityId) -> T {
        let component = (self.build_fn)(world, entity);
        Modifier::call(&self.modifiers, component, world, entity)
    }

    /// Map the component `T` to another component `U` using the given function `f` during build.
    #[inline]
    #[must_use]
    pub fn map<U, F>(self, f: F) -> Proto<U>
    where
        F: Fn(T) -> U + Send + Sync + 'static,
        T: 'static,
        U: 'static,
    {
        Proto::new(move |world, entity| {
            let t = self.build_component(world, entity);
            f(t)
        })
    }

    /// Add a modifier function `f` applying changes to the component `T` after building
    #[inline]
    #[must_use]
    pub fn with<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut T) + Send + Sync + 'static,
        T: 'static,
    {
        let modifier = Modifier::With(Arc::new(f));
        self.modifiers.push(modifier);
        self
    }

    /// Add a modifier function `f` applying changes to the component `T` after building, with
    /// access to scene context
    #[inline]
    #[must_use]
    pub fn with_world<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut T, &mut World, EntityId) + Send + Sync + 'static,
        T: 'static,
    {
        let modifier = Modifier::WithWorld(Arc::new(f));
        self.modifiers.push(modifier);
        self
    }

    /// Add a modifier function `f` which takes ownership of the component `T` and returns a new
    /// `T` after building
    #[inline]
    #[must_use]
    pub fn with_take<F>(mut self, f: F) -> Self
    where
        F: Fn(T) -> T + Send + Sync + 'static,
        T: 'static,
    {
        let modifier = Modifier::WithTake(Arc::new(f));
        self.modifiers.push(modifier);
        self
    }

    /// Conditionally add a [`with`](Proto::with) modifier if `condition` is true
    #[inline]
    #[must_use]
    pub fn with_if<F>(mut self, condition: bool, f: F) -> Self
    where
        F: Fn(&mut T) + Send + Sync + 'static,
        T: 'static,
    {
        if condition {
            let modifier = Modifier::With(Arc::new(f));
            self.modifiers.push(modifier);
        }
        self
    }
}

impl<T: Component> Scene for Proto<T> {
    fn build(&self, world: &mut World, entity: EntityId) {
        let component = self.build_component(world, entity);
        world.insert_component(entity, component, false);
    }
}
