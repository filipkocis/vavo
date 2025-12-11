use std::any::Any;

use crate::{
    assets::Scene,
    prelude::{EntityId, World},
};

#[derive(Default)]
/// A list of scene objects to be built into the ECS world.
/// Describes a node and its components, as well as children.
pub struct SceneList {
    /// List of scene objects to build
    scenes: Vec<Box<dyn Scene>>,
    /// Whether to create this node as a child of the parent entity
    child: bool,
}

impl SceneList {
    /// Create a scene list for a child entity
    #[inline]
    pub fn child() -> Self {
        Self {
            child: true,
            ..Self::default()
        }
    }

    /// Check if this scene list is for a child entity
    #[inline]
    pub fn is_child(&self) -> bool {
        self.child
    }

    /// Push a new scene object into the list
    #[inline]
    pub fn push<S: Scene>(&mut self, scene: S) {
        self.push_boxed(Box::new(scene));
    }

    /// Internal push logic for `push`
    #[inline]
    fn push_boxed(&mut self, mut scene: Box<dyn Scene>) {
        let new_any = scene.as_mut() as &mut dyn Any;
        let new_type_id = (*new_any).type_id();

        if let Some(list) = new_any.downcast_mut::<SceneList>() {
            if list.child {
                // Just add the SceneList as a child
                self.scenes.push(scene);
            } else {
                // Unwrap nested SceneList and merge with this one
                for nested_scene in list.scenes.drain(..) {
                    self.push_boxed(nested_scene);
                }
            }

            return;
        }

        // Replace existing scene objects of the same type
        for current in &mut self.scenes {
            let curr_any = current.as_ref() as &dyn Any;
            let curr_type_id = (*curr_any).type_id();

            if curr_type_id == new_type_id {
                *current = scene;
                return;
            }
        }

        // Not a duplicate, not a list, just add it
        self.scenes.push(scene);
    }
}

impl Scene for SceneList {
    fn build(&self, world: &mut World, entity: EntityId) {
        for scene in self.scenes.iter().rev() {
            if self.child {
                // Create a new child entity
                let child_entity = world.spawn();
                scene.build(world, child_entity);

                // Attach to parent
                world.add_child(entity, child_entity);
            } else {
                // Apply normally
                scene.build(world, entity);
            }
        }
    }
}

/// Create a scene from any components implementing [Scene], you can use the
/// [`proto`](SceneProto::proto) method, or the [Default] implementation for your components.
/// What you pass to this macro must implement `scene` trait, then you can [build](Scene::build) it
/// into the ECS world.
///
/// If you want to create child entity hierarchies, use the [`children!`](children) macro. Both
/// macros can be nested within each other because they return a [SceneList]. There is also a
/// [`child!`](child) macro for creating just a single child entity.
///
/// Check out [SceneProto] if you want to implement the `scene` trait.
///
/// # Usage
/// Same syntax as the `vec!` macro, separate components by commas:
/// ```rust
/// # use vavo::prelude::*;
/// // Create an entity with two components
/// let my_scene = scene![
///     Name::new("My Entity"),
///     Transform::proto(),
/// ];
/// ```
/// Children are combined, not overriden:
/// ```rust
/// # use vavo::prelude::*;
/// // Create an entity with four children
/// let my_scene = scene![
///     child![
///         Name::new("Child 1"),
///         // Nested child/children here are **not** unwrapped
///         child![Name::new("Grandchild 1")],
///     ],
///
///     children![
///         // Local child
///         (Name::new("Child 2")),
///
///         // Nested child/children here are **not** unwrapped
///         (child![Name::new("Child 3")]),
///         (children![(Name::new("Child 4"))])
///     ],
/// ];
/// ```
/// You can also inherit components from other scenes by nesting them, they will be built in order,
/// with later components overriding earlier ones:
/// ```rust
/// # use vavo::prelude::*;
/// // Base scene used for inheritance
/// fn base_scene() -> impl Scene {
///     scene![
///         Name::new("Base Entity"),
///         Transform::proto(),
///     ]
/// }
///
/// // Scene constructed here will be unwrapped and built in order
/// let my_scene = scene![
///     base_scene(), // Inherit base scene components
///     Name::new("Root"),
///     scene![
///         Name::new("Override"),
///         scene![
///             Name::new("Deep Override"),
///         ]
///     ],
/// ];
///
/// // The above is equivalent to:
/// let equivalent_scene = scene![
///     Name::new("Deep Override"), // Overrides all because it's last
///     Transform::proto(), // from base_scene
/// ];
///
/// ```
#[macro_export]
macro_rules! scene {
    ($($component:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut list = $crate::assets::scene::SceneList::default();
        $(
            list.push($component);
        )*
        list
    }}
}

/// [`scene!`](scene) macro for creating multiple children for an entity. Each child must be a
/// tuple of comma separated components
/// # Usage
/// ```rust
/// # use vavo::prelude::*;
/// children![
///     // Tuple of components per child
///     (Name::new("Child 1")),
///     (Name::new("Child 2"), Transform::proto()),
///
///     // Nested child/children will act as a new child with children, it does not unwrap
///     (child![Name::new("Child 3")]),
///     (children![
///         (Name::new("Child 4")),
///     ]),
/// ];
#[macro_export]
macro_rules! children {
    ($(( $( $component:expr ),+ )),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut children = $crate::assets::scene::SceneList::default();
        $(
            let child = $crate::child![ $( $component ),+ ];
            children.push(child);
        )*
        children
    }}
}

/// [`scene!`](scene) macro for creating a single child for an entity, each component must be comma
/// separated, just like the `vec!` macro
#[macro_export]
macro_rules! child {
    ($($component:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut list = $crate::assets::scene::SceneList::child();
        $(
            list.push($component);
        )*
        list
    }}
}
