//! This module manages the culling of objects in the scene during rendering.
//! Currently, it only implements frustum culling.
//!
//! By default, each entity with a mesh component will have [`LocalBoundingVolume::Sphere`] added to,
//! currently it doesn't get recalculated on mesh change, or re-added. It does however get
//! recalculated if you change the volume type with `to_**` methods.
//!
//! Every component with `local bounding volume` will have [`Visibility`] and
//! [`WorldBoundingVolume`] component added to it, in case it's removed it will be added again.  
//! These are recalculated on `GlobalTransform` or `LocalBoundingVolume` change.
//!
//! First camera in the scene will have a [`Frustum`] component added to it, if the `Frustum` or the
//! `GlobalTransform` of the camera changes, all objects will have their `Visibility` recalculated.
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
            .register_system(visibility_update_system, SystemStage::Render);
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

/// This system gets entities with `local bounding volume` where either `GlobalTransform` or
/// `LocalBoundingVolume` has changed, and updates the `WorldBoundingVolume` and `Visibility`.
/// If they are not present, they will be added, hence the `Option<&mut T>` in the query.
pub fn visibility_update_system(
    ctx: &mut SystemsContext,
    mut query: Query<
        (
            EntityId,
            &LocalBoundingVolume,
            Option<&mut WorldBoundingVolume>,
            &GlobalTransform,
            Option<&mut Visibility>,
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

    // // check if the camera has changed
    // let changed_camera = query
    //     .cast::<EntityId, Or<(Changed<GlobalTransform>, Changed<Frustum>)>>()
    //     .get(camera_id);
    //
    // let final_query;
    // if changed_camera.is_some() {
    //     // if the camera has changed, we need to update all objects
    //     final_query = query
    //         .cast::<(
    //             &LocalBoundingVolume,
    //             Option<&mut WorldBoundingVolume>,
    //             &GlobalTransform,
    //             &mut Visibility,
    //         ), ()>()
    //         .iter_mut()
    // } else {
    //     final_query = query.iter_mut();
    // }

    for (id, local_bv, world_bv, global_transform, visibility) in query.iter_mut() {
        let visible;

        if let Some(world_bv) = world_bv {
            // update world bounding volume
            *world_bv = local_bv.to_world_space(&global_transform.matrix);

            // check for intersections
            visible = frustum.intersects(world_bv);
        } else {
            // create world bounding volume
            let world_bv = local_bv.to_world_space(&global_transform.matrix);

            // check for intersections
            visible = frustum.intersects(&world_bv);

            // add component
            ctx.commands.entity(id).insert(world_bv);
        }

        // update visibility
        if let Some(visibility) = visibility {
            visibility.visible = visible;
        } else {
            ctx.commands.entity(id).insert(Visibility::new(visible));
        }
    }
}
