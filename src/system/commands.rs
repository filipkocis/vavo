use std::any::{Any, TypeId};

use crate::{math::{GlobalTransform, Transform}, ecs::{resources::Resource, world::World, entities::EntityId}};

enum Command {
    InsertResource(TypeId, Box<dyn Any>),
    RemoveResource(TypeId),
    SpawnEntity(EntityId),
    DespawnEntity(EntityId),
    DespawnEntityRecursive(EntityId),
    InsertComponent(EntityId, Box<dyn Any>),
    RemoveComponent(EntityId, TypeId),
    AddChild(EntityId, EntityId),
    RemoveChild(EntityId, EntityId),
}

pub struct Commands {
    next_entity_id: EntityId,
    commands: Vec<Command>,
}

pub struct EntityCommands<'a> {
    entity_id: EntityId,
    commands: &'a mut Commands,
}

pub struct ParentCommands<'a> {
    parent_id: EntityId,
    commands: &'a mut Commands,
}

impl<'a> ParentCommands<'a> {
    fn new(parent_id: EntityId, commands: &'a mut Commands) -> Self {
        Self { 
            parent_id,
            commands,
        }
    }

    pub fn spawn_empty(&mut self) -> EntityCommands {
        let child_id = self.commands.spawn_empty().entity_id;

        self.commands
            .commands
            .push(Command::AddChild(self.parent_id, child_id));

        let mut child = EntityCommands::new(self.commands);
        child.set_entity_id(child_id);
        child
    }
}

impl<'a> EntityCommands<'a> {
    fn new(commands: &'a mut Commands) -> Self {
        Self {
            entity_id: commands.next_entity_id - 1,
            commands,
        }
    }

    fn set_entity_id(&mut self, entity_id: EntityId) {
        self.entity_id = entity_id;
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn despawn(self) {
        self.commands
            .commands
            .push(Command::DespawnEntity(self.entity_id));
    }

    pub fn despawn_recursive(self) {
        self.commands
            .commands
            .push(Command::DespawnEntityRecursive(self.entity_id));
    }

    pub fn insert<T: 'static>(mut self, component: T) -> Self {
        self.handle_insert_types(&component);
        self.insert_internal(component);
        self
    }

    pub fn remove<T: 'static>(self) -> Self {
        self.commands
            .commands
            .push(Command::RemoveComponent(self.entity_id, TypeId::of::<T>()));
        self
    }

    pub fn with_children<F: FnOnce(&mut ParentCommands)>(mut self, f: F) -> Self {
        let mut parent_commands = ParentCommands::new(self.entity_id, &mut self.commands);
        f(&mut parent_commands);
        self
    }

    pub fn remove_children(self, children: Vec<EntityId>) -> Self {
        for child_id in children {
            self.commands
                .commands
                .push(Command::RemoveChild(self.entity_id, child_id));
        }
        self
    }

    pub fn insert_children(self, children: Vec<EntityId>) -> Self {
        for child_id in children {
            self.commands
                .commands
                .push(Command::AddChild(self.entity_id, child_id));
        }
        self
    }

    fn insert_internal<T: 'static>(&mut self, component: T) {
        self.commands.commands.push(Command::InsertComponent(
            self.entity_id,
            Box::new(component),
        ));
    }

    fn handle_insert_types<T: 'static>(&mut self, component: &T) {
        let type_id = TypeId::of::<T>();

        if type_id == TypeId::of::<EntityId>() {
            panic!("Cannot insert EntityId component");
        } else if type_id == TypeId::of::<GlobalTransform>() {
            panic!("Cannot insert GlobalTransform component");
        }

        if type_id == TypeId::of::<Transform>() {
            let transform = component as *const T as *const Transform;
            self.insert_internal(GlobalTransform::from_transform(unsafe { &*transform }));
        }
    }
}

impl Commands {
    pub(crate) fn build(world: &World) -> Self {
        Self {
            next_entity_id: world.entities.next_entity_id(),
            commands: Vec::new(),
        }
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        let boxed_resource = Box::new(resource);
        let resource_type_id = TypeId::of::<R>();

        self.commands
            .push(Command::InsertResource(resource_type_id, boxed_resource));
        self
    }

    pub fn remove_resource<R: Resource>(&mut self) -> &mut Self {
        self.commands
            .push(Command::RemoveResource(TypeId::of::<R>()));
        self
    }

    pub fn spawn_empty(&mut self) -> EntityCommands {
        self.commands
            .push(Command::SpawnEntity(self.next_entity_id));
        self.next_entity_id = self.next_entity_id + 1;

        EntityCommands::new(self)
    }

    pub fn entity(&mut self, entity_id: EntityId) -> EntityCommands {
        let mut entity_commands = EntityCommands::new(self);
        entity_commands.set_entity_id(entity_id);
        entity_commands
    }

    pub(crate) fn apply(self, world: &mut World) {
        for command in self.commands {
            match command {
                Command::InsertResource(type_id, resource) => {
                    world.resources.insert_boxed(type_id, resource);
                }
                Command::RemoveResource(type_id) => {
                    world.resources.remove(type_id);
                }
                Command::SpawnEntity(entity_id) => {
                    world.entities.spawn_entity(entity_id, Vec::new());
                }
                Command::DespawnEntity(entity_id) => {
                    world.entities.despawn_entity(entity_id);
                }
                Command::DespawnEntityRecursive(entity_id) => {
                    world.entities.despawn_entity_recursive(entity_id);
                }
                Command::InsertComponent(entity_id, component) => {
                    world.entities.insert_component(entity_id, component);
                }
                Command::RemoveComponent(entity_id, type_id) => {
                    world.entities.remove_component(entity_id, type_id);
                }
                Command::AddChild(parent_id, child_id) => {
                    world.entities.add_child(parent_id, child_id);
                }
                Command::RemoveChild(parent_id, child_id) => {
                    world.entities.remove_child(parent_id, child_id);
                }
            }
        }

        // world.entities.set_next_entity_id(self.next_entity_id);
        assert_eq!(world.entities.next_entity_id(), self.next_entity_id);
    }
}
