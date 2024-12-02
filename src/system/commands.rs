use std::any::{Any, TypeId};

use crate::world::{World, entities::EntityId};

enum Command {
    InsertResource(Box<dyn Any>),
    RemoveResource(TypeId),
    SpawnEntity(EntityId),
    DespawnEntity(EntityId),
    InsertComponent(EntityId, Box<dyn Any>),
    RemoveComponent(EntityId, TypeId),
}

pub struct Commands {
    next_entity_id: EntityId,
    commands: Vec<Command>,
}

pub struct EntityCommands<'a> {
    entity_id: EntityId,
    commands: &'a mut Commands,
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

    pub fn despawn(self) {
        self.commands
            .commands
            .push(Command::DespawnEntity(self.entity_id));
    }

    pub fn insert<T: 'static>(self, component: T) -> Self {
        self.commands.commands.push(Command::InsertComponent(
            self.entity_id,
            Box::new(component),
        ));
        self
    }

    pub fn remove<T: 'static>(self) -> Self {
        self.commands
            .commands
            .push(Command::RemoveComponent(self.entity_id, TypeId::of::<T>()));
        self
    }
}

impl Commands {
    pub(crate) fn build(world: &World) -> Self {
        Self {
            next_entity_id: world.entities.next_entity_id(),
            commands: Vec::new(),
        }
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) -> &mut Self {
        self.commands
            .push(Command::InsertResource(Box::new(resource)));
        self
    }

    pub fn remove_resource<T: 'static>(&mut self) -> &mut Self {
        self.commands
            .push(Command::RemoveResource(TypeId::of::<T>()));
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
                Command::InsertResource(resource) => {
                    world.resources.insert(resource);
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
                Command::InsertComponent(entity_id, component) => {
                    world.entities.insert_component(entity_id, component);
                }
                Command::RemoveComponent(entity_id, type_id) => {
                    world.entities.remove_component(entity_id, type_id);
                }
            }
        }

        // world.entities.set_next_entity_id(self.next_entity_id);
        assert_eq!(world.entities.next_entity_id(), self.next_entity_id);
    }
}
