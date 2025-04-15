//! This module manages the culling of objects in the scene during rendering.
//! Currently, it only implements frustum culling.
//!
//! For settings, see [`FrustumCullingSettings`].
//!
//! By default, each entity with a mesh component will have [`LocalBoundingVolume::Sphere`], and
//! default `WorldBoundingVolume` and `Visibility` components added to it. If any of these get
//! removed, they will be readded. Currently, changes on mesh or LBV won't trigger a recalculation.
//! Only a direct change in response to `Query<&mut Handle<Mesh>>` will trigger it.
//!
//! Every entity with [`LocalBoundingVolume`], [`Visibility`] and [`WorldBoundingVolume`]
//! components will have their WBV and Visibility recalculated on `GlobalTransform` or 
//! `LocalBoundingVolume` change.
//!
//! All active cameras in the scene will have a [`Frustum`] component added to it, it will be
//! recalculated on `GlobalTransform` change. If the camera's `Frustum` changes, all entities will
//! have their `Visibility` recalculated.
//!
//! For more information, see [`FrustumCullingPlugin`].

use crate::{
    math::bounding_volume::{
        Frustum, LocalBoundingVolume, Sphere, ToWorldSpace, WorldBoundingVolume,
    },
    prelude::*,
};

/// This plugin adds resources and systems for frustum culling. For more information, see the
/// [culling module](crate::renderer::culling).
pub struct FrustumCullingPlugin;

impl Plugin for FrustumCullingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrustumCullingSettings>()
            // These two use `commands.insert`, so we need them in separate stages to apply
            .register_system(add_local_bounding_volume_system, SystemStage::PostUpdate)
            .register_system(update_camera_frustum_system, SystemStage::Last)
            // TODO: since GlobalTransform is updated in the Last stage we have to move them up, fix
            // this after Changed<T> acts differently, originally it was in the PostUpdate
            .register_system(visibility_update_system, SystemStage::PreRender)
            .register_system(frustum_visibility_update_system, SystemStage::PreRender);
    }
}

#[derive(Resource)]
/// Settings used for frustum culling. Used as a resource.
pub struct FrustumCullingSettings {
    /// Wheter to use frustum culling
    pub enabled: bool,
}

impl Default for FrustumCullingSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Component)]
/// This component indicates whether an entity is visible in the frustum.
/// Shouldn't be used directly, it's used as an internal cache for the culling system.
pub struct Visibility {
    pub visible: bool,
}

impl Visibility {
    pub fn new(visible: bool) -> Self {
        Self { visible }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

/// This system updates the `Visibility` component of all entities in the scene if the camera has
/// its `Frustum` changed.
pub fn frustum_visibility_update_system(
    ctx: &mut SystemsContext,
    mut query: Query<(&WorldBoundingVolume, &mut Visibility)>,
) {
    // early exit based on settings
    let settings = ctx.resources.get::<FrustumCullingSettings>().unwrap();
    if !settings.enabled {
        return;
    }

    // extract the active camera
    let active_camera = query
        .cast::<(&Camera, &Frustum), Changed<Frustum>>()
        .iter_mut()
        .into_iter()
        .find(|(camera, _)| camera.active);

    let Some((_, frustum)) = active_camera else {
        return;
    };

    for (world_bv, visibility) in query.iter_mut() {
        // check for intersections
        let visible = frustum.intersects(world_bv);

        // update visibility
        visibility.visible = visible;
    }
}

/// This system updates the `Frustum` component of all active cameras in the scene based on
/// `GlobalTransform` or `Projection` change. The component is added if it doesn't exist.
pub fn update_camera_frustum_system(
    ctx: &mut SystemsContext,
    mut query: Query<
        (
            EntityId,
            &Camera,
            &Projection,
            &GlobalTransform,
            Option<&mut Frustum>,
        ),
        Or<(
            Changed<GlobalTransform>,
            Changed<Projection>,
            Without<Frustum>,
        )>,
    >,
) {
    // early exit based on settings
    let settings = ctx.resources.get::<FrustumCullingSettings>().unwrap();
    if !settings.enabled {
        return;
    }

    for (id, camera, projection, global_transform, frustum) in query.iter_mut() {
        if !camera.active {
            continue;
        }

        // calculate the frustum
        let planes = projection.get_frustum_planes(&global_transform.matrix);
        let new_frustum = Frustum::new(planes);

        // update frustum
        if let Some(frustum) = frustum {
            *frustum = new_frustum;
        } else {
            ctx.commands.entity(id).insert(new_frustum);
        }
    }
}

/// This system (re)adds a `LocalBoundingVolume::Sphere` to all entities with a `Mesh` component.
/// It also adds default `WorldBoundingVolume::None` and `Visibility::new(false)`
pub fn add_local_bounding_volume_system(
    ctx: &mut SystemsContext,
    mut query: Query<
        (EntityId, &Handle<Mesh>),
        Or<(
            Without<LocalBoundingVolume>,
            Without<WorldBoundingVolume>,
            Without<Visibility>,
            Changed<Handle<Mesh>>,
        )>,
    >,
) {
    // early exit based on settings
    let settings = ctx.resources.get::<FrustumCullingSettings>().unwrap();
    if !settings.enabled {
        return;
    }

    let mesh_assets = ctx.resources.get::<Assets<Mesh>>().unwrap();
    for (id, mesh_handle) in query.iter_mut() {
        // get the mesh
        let mesh = mesh_assets.get(mesh_handle).unwrap();

        // add the local bounding volume
        let sphere = Sphere::from_mesh(mesh);
        ctx.commands
            .entity(id)
            .insert(LocalBoundingVolume::Sphere(sphere))
            .insert(WorldBoundingVolume::None)
            .insert(Visibility::new(false));
    }
}

/// This system gets entities with `local bounding volume` where either `GlobalTransform` or
/// `LocalBoundingVolume` has changed, and updates the `WorldBoundingVolume` and `Visibility`.
pub fn visibility_update_system(
    ctx: &mut SystemsContext,
    mut query: Query<
        (
            &LocalBoundingVolume,
            &mut WorldBoundingVolume,
            &GlobalTransform,
            &mut Visibility,
        ),
        Or<(Changed<GlobalTransform>, Changed<LocalBoundingVolume>)>,
    >,
) {
    // early exit based on settings
    let settings = ctx.resources.get::<FrustumCullingSettings>().unwrap();
    if !settings.enabled {
        return;
    }

    // extract the active camera
    let active_camera = query
        .cast::<(&Camera, &Frustum), ()>()
        .iter_mut()
        .into_iter()
        .find(|(camera, _)| camera.active);

    let Some((_, frustum)) = active_camera else {
        return;
    };

    for (local_bv, world_bv, global_transform, visibility) in query.iter_mut() {
        // update world bounding volume
        *world_bv = local_bv.to_world_space(&global_transform.matrix);

        // check for intersections
        let visible = frustum.intersects(world_bv);

        // update visibility
        visibility.visible = visible;
    }
}
