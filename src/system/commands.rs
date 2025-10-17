use std::{any::TypeId, mem::ManuallyDrop};

use crate::{
    ecs::{
        entities::{tracking::EntityTracking, Component, EntityId},
        ptr::OwnedPtr,
        resources::{Resource, ResourceData},
        tick::Tick,
        world::World,
    },
    math::{GlobalTransform, Transform},
    prelude::{Children, Parent},
};

/// Command to be executed on the world.
enum Command {
    InsertResource(TypeId, ResourceData),
    RemoveResource(TypeId),
    SpawnEntity(EntityId),
    DespawnEntity(EntityId),
    DespawnEntityRecursive(EntityId),
    InsertComponent(Box<dyn FnOnce(&mut World)>),
    RemoveComponent(EntityId, TypeId),
    AddChild(EntityId, EntityId),
    RemoveChild(EntityId, EntityId),
}

/// Queue of commands to be applied to the world.
pub struct Commands {
    tracking: *mut EntityTracking,
    commands: Vec<Command>,
}

/// Commands for a specific entity.
pub struct EntityCommands<'a> {
    entity_id: EntityId,
    commands: &'a mut Commands,
}

/// Commands for creating child entities under a parent.
pub struct ParentCommands<'a> {
    parent_id: EntityId,
    commands: &'a mut Commands,
}

impl<'a> ParentCommands<'a> {
    /// Creates new parent commands
    fn new(parent_id: EntityId, commands: &'a mut Commands) -> Self {
        Self {
            parent_id,
            commands,
        }
    }

    /// Spawns a new empty child entity under the parent and returns its [`EntityCommands`].
    pub fn spawn_empty(&mut self) -> EntityCommands<'_> {
        let child_id = self.commands.spawn_empty().entity_id;

        self.commands
            .commands
            .push(Command::AddChild(self.parent_id, child_id));

        EntityCommands::new(self.commands, child_id)
    }
}

impl<'a> EntityCommands<'a> {
    /// Creates new entity commands
    #[inline]
    fn new(commands: &'a mut Commands, entity_id: EntityId) -> Self {
        Self {
            entity_id,
            commands,
        }
    }

    /// Returns the id of the entity.
    #[inline]
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Despawn the entity and break its parent-child relationship.
    pub fn despawn(self) {
        self.commands
            .commands
            .push(Command::DespawnEntity(self.entity_id));
    }

    /// Despawns the entity and all its children recursively.
    pub fn despawn_recursive(self) {
        self.commands
            .commands
            .push(Command::DespawnEntityRecursive(self.entity_id));
    }

    /// Inserts new component to the entity.
    pub fn insert<C: Component>(mut self, component: C) -> Self {
        self.handle_insert_types(&component, true);
        self.insert_internal(component, true);
        self
    }

    /// Inserts new component to the entity if the condition returns true.
    pub fn insert_if<C: Component, F: FnOnce() -> bool>(self, component: C, condition: F) -> Self {
        if condition() {
            self.insert(component)
        } else {
            self
        }
    }

    /// Inserts new component to the entity if it doesn't exist.
    pub fn insert_if_new<C: Component>(mut self, component: C) -> Self {
        self.handle_insert_types(&component, false);
        self.insert_internal(component, false);
        self
    }

    /// Inserts new component to the entity if it doesn't exist, and if the condition returns true.
    pub fn insert_if_new_if<C: Component, F: FnOnce() -> bool>(
        self,
        component: C,
        condition: F,
    ) -> Self {
        if condition() {
            self.insert_if_new(component)
        } else {
            self
        }
    }

    /// Removes a component from the entity.
    pub fn remove<C: Component>(self) -> Self {
        self.commands
            .commands
            .push(Command::RemoveComponent(self.entity_id, TypeId::of::<C>()));
        self
    }

    /// Takes a closure in which you can create new child entities.
    pub fn with_children<F: FnOnce(&mut ParentCommands)>(self, f: F) -> Self {
        let mut parent_commands = ParentCommands::new(self.entity_id, self.commands);
        f(&mut parent_commands);
        self
    }

    /// Removes all children in the list from the entity.
    pub fn remove_children(self, children: Vec<EntityId>) -> Self {
        for child_id in children {
            self.commands
                .commands
                .push(Command::RemoveChild(self.entity_id, child_id));
        }
        self
    }

    /// Inserts already existing children to the entity.
    pub fn insert_children(self, children: Vec<EntityId>) -> Self {
        for child_id in children {
            self.commands
                .commands
                .push(Command::AddChild(self.entity_id, child_id));
        }
        self
    }

    /// Inserts an already existing child to the entity.
    pub fn insert_child(self, child: EntityId) -> Self {
        self.commands
            .commands
            .push(Command::AddChild(self.entity_id, child));
        self
    }

    #[inline]
    /// Inserts a new component
    fn insert_internal<C: Component>(&mut self, component: C, replace: bool) {
        let entity_id = self.entity_id;

        let insert_closure = move |world: &mut World| {
            let info = world.registry.get_or_register::<C>();
            let mut component = ManuallyDrop::new(component);
            // Safety: component is inserted and not used anymore
            let ptr = unsafe { OwnedPtr::new_ref(&mut component) };

            world
                .entities
                .insert_component(entity_id, ptr, info, replace);
        };

        self.commands
            .commands
            .push(Command::InsertComponent(Box::new(insert_closure)))
    }

    /// Checks and handles special cases of the component being inserted
    fn handle_insert_types<C: Component>(&mut self, component: &C, replace: bool) {
        let type_id = TypeId::of::<C>();

        if type_id == TypeId::of::<EntityId>() {
            panic!("Cannot insert EntityId component");
        } else if type_id == TypeId::of::<GlobalTransform>() {
            panic!("Cannot insert GlobalTransform component");
        }

        if type_id == TypeId::of::<Transform>() {
            let transform = component as *const C as *const Transform;
            self.insert_internal(
                GlobalTransform::from_transform(unsafe { &*transform }),
                replace,
            );
        }
    }
}

impl Commands {
    /// Creates new command queue.
    /// Provide `next_entity_id` from [`Entities::next_entity_id`](crate::prelude::Entities::next_entity_id)
    /// Safety: `tracking` must live as long as the `Commands` instance and must not be aliased
    /// while `Commands` exists.
    pub fn build(tracking: &mut EntityTracking) -> Self {
        Self {
            tracking,
            commands: Vec::new(),
        }
    }

    /// Inserts or replaces a resource of type `R` in the world.
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        let resource_data = ResourceData::new(resource, Tick::default());
        let resource_type_id = TypeId::of::<R>();

        self.commands
            .push(Command::InsertResource(resource_type_id, resource_data));
        self
    }

    /// Removes a resource of type `R` from the world.
    pub fn remove_resource<R: Resource>(&mut self) -> &mut Self {
        self.commands
            .push(Command::RemoveResource(TypeId::of::<R>()));
        self
    }

    /// Spawns a new empty entity and returns its [`EntityCommands`] to modify it.
    pub fn spawn_empty(&mut self) -> EntityCommands<'_> {
        let new_id = self.tracking().new_id();
        self.commands.push(Command::SpawnEntity(new_id));

        EntityCommands::new(self, new_id)
    }

    /// Selects an entity and returns its [`EntityCommands`] to modify it.
    #[inline]
    pub fn entity(&mut self, entity_id: EntityId) -> EntityCommands<'_> {
        EntityCommands::new(self, entity_id)
    }

    /// Applies all queued commands to the world.
    pub fn apply(&mut self, world: &mut World) {
        for command in self.commands.drain(..) {
            match command {
                Command::InsertResource(type_id, mut resource_data) => {
                    resource_data.set_tick(*world.tick);
                    world.resources.insert_resource_data(type_id, resource_data);
                }
                Command::RemoveResource(type_id) => {
                    world.resources.remove(type_id);
                }
                Command::SpawnEntity(entity_id) => {
                    let entity_info = world.registry.get_or_register::<EntityId>();
                    world
                        .entities
                        .spawn_entity(entity_id, Vec::new(), entity_info);
                }
                Command::DespawnEntity(entity_id) => {
                    world.entities.despawn_entity(entity_id);
                }
                Command::DespawnEntityRecursive(entity_id) => {
                    world.entities.despawn_entity_recursive(entity_id);
                }
                Command::InsertComponent(insert_closure) => {
                    insert_closure(world);
                }
                Command::RemoveComponent(entity_id, type_id) => {
                    world.entities.remove_component(entity_id, type_id);
                }
                Command::AddChild(parent_id, child_id) => {
                    let parent_info = world.registry.get_or_register::<Parent>();
                    let children_info = world.registry.get_or_register::<Children>();
                    world
                        .entities
                        .add_child(parent_id, child_id, parent_info, children_info);
                }
                Command::RemoveChild(parent_id, child_id) => {
                    world.entities.remove_child(parent_id, child_id);
                }
            }
        }
    }

    /// Returns a mutable reference to the entity tracking system.
    #[inline]
    fn tracking(&mut self) -> &mut EntityTracking {
        unsafe { &mut *self.tracking }
    }
}
