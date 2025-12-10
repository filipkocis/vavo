use crate::{
    assets::Scene,
    prelude::{EntityId, World},
};

#[derive(Default)]
/// A list of scene objects to be built into the ECS world.
/// Describes a node and its components, as well as children.
pub struct SceneList {
    pub scenes: Vec<Box<dyn Scene>>,
    /// Whether to create this node as a child of the parent entity
    pub child: bool,
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

    /// Push a new scene object into the list
    #[inline]
    pub fn push(&mut self, scene: impl Scene) {
        self.scenes.push(Box::new(scene));
    }
}

impl Scene for SceneList {
    fn build(&self, world: &mut World, entity: EntityId) {
        // Since replace is false, we need to build from the end
        for scene in self.scenes.iter().rev() {
            scene.build(world, entity);
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
///         // Nested child in children is unwrapped
///         (child![Name::new("Child 3")]),
///
///         // Nested children in children are unwrapped
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
        let list = vec![ $( Box::new($component) as Box<dyn $crate::assets::scene::Scene>, )* ];
        $crate::assets::scene::SceneList {
            scenes: list,
            child: false,
        }
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
///     // You can also use the `child!` macro
///     child![Name::new("Child 3")],
///
///     // Nested children of children are `unwrapped` since it doesnt belong to a entity directly
///     children![
///         (Name::new("Child 4")),
///     ]
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
        let list = vec![ $( Box::new($component) as Box<dyn $crate::assets::scene::Scene>, )* ];
        $crate::assets::scene::SceneList {
            scenes: list,
            child: true,
        }
    }}
}
