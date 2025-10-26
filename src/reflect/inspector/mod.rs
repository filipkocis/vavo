use std::any::TypeId;

use crate::{prelude::*, ui::prelude::*};

/// Provides a Inspector Tool for dynamic reflection of types.
pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.register_state::<InspectorState>()
            .add_startup_system(setup_inspector)
            .add_system(handle_inspector)
            .add_system(create_inspector.run_if(on_enter(InspectorState::On)))
            .add_system(cleanup_inspector.run_if(on_exit(InspectorState::On)));
    }
}

#[derive(Component)]
struct InspectorMenu;

#[derive(States, Default, Debug, PartialEq, Eq, Clone, Copy)]
enum InspectorState {
    On,
    #[default]
    Off,
}

/// Sets up resources for Inspector
fn setup_inspector() {}

/// Handles the input for the Inspector menu
fn handle_inspector(
    input: Res<Input<KeyCode>>,
    state: Res<State<InspectorState>>,
    mut next_state: ResMut<NextState<InspectorState>>,
) {
    if input.just_pressed(KeyCode::Backquote) {
        match state.get() {
            InspectorState::On => next_state.set(InspectorState::Off),
            InspectorState::Off => next_state.set(InspectorState::On),
        }
    }

    if state.get() == InspectorState::On && input.just_pressed(KeyCode::Escape) {
        next_state.set(InspectorState::Off);
    }
}

/// Creates the Inspector UI menu
fn create_inspector(
    mut commands: Commands,
    mut query: Query<(EntityId, &Transform, &GlobalTransform)>,
    app: &mut App,
) {
    let query_result = query.iter_mut();
    let count = query_result.len();

    let menu = commands
        .spawn_empty()
        .insert(InspectorMenu)
        .insert(Node {
            border: UiRect::all(Val::Px(2.0)),
            border_color: color::RED,
            background_color: Color::new(0.0, 0.0, 0.0, 0.8),
            ..Default::default()
        })
        .entity_id();

    commands.entity(menu).with_children(|p| {
        p.spawn_empty()
            .insert(Node {
                color: Some(color::WHITE),
                background_color: color::TRANSPARENT,
                ..Default::default()
            })
            .insert(Text::new(format!("total: {:?}", count)));
    });

    for (id, transform, global) in query_result {
        commands.entity(menu).with_children(|p| {
            p.spawn_empty()
                .insert(Node {
                    color: Some(color::WHITE),
                    background_color: color::TRANSPARENT,
                    ..Default::default()
                })
                .insert(Text::new(format!("{:?}:", id.index())));
        });
    }

    let registry = &app.type_registry;
    println!("PRINTING");
    for archetype in app.world.entities.archetypes() {
        let id_idx = archetype.component_index(&TypeId::of::<EntityId>());

        for entity in 0..archetype.len() {
            let mut entity_id = None;
            let components: Vec<_> = archetype
                .components
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    if i == id_idx {
                        entity_id = registry
                            .reflect(c.get_untyped_lt(entity), c.get_type_id())
                            .unwrap()
                            .downcast_ref::<EntityId>();
                        return None;
                    }
                    registry.reflect(c.get_untyped_lt(entity), c.get_type_id())
                })
                .collect();

            print!("{:?}: ", entity_id.unwrap().to_bits());
            for c in components {
                print!("{:?} ", c);
            }
            println!();
        }
    }
}

/// Despawns Inspector UI menu
fn cleanup_inspector(mut commands: Commands, mut query: Query<EntityId, With<InspectorMenu>>) {
    if let Some(id) = query.iter_mut().first() {
        commands.entity(*id).despawn_recursive();
    }
}
