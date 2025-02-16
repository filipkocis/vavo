use std::time::Duration;

use kira::{listener::{ListenerHandle, ListenerId}, Tween};

use crate::prelude::*;

use super::{AudioManager, SystemsContext};

/// A marker component used to specify which [`entity`](EntityId) is the spatial listener, it's not inserted
/// automatically, you have to insert it manually. Most likely you will want to attach it to the
/// [`camera`](Camera) entity. 
///
/// Spatial listener automatically tracks the position and the orientation of the entity it is
/// attached to.
/// 
/// [`AudioTrack`](AudioTrack) uses the first spatial listener it finds, so
/// more than one spatial listeners are useless.
#[derive(Component, Default, Debug)]
pub struct SpatialListener(pub(crate) Option<ListenerHandle>);

impl SpatialListener {
    /// Returns the id of the spatial listener
    pub(crate) fn id(&self) -> Option<ListenerId> {
        self.0.as_ref().map(|handle| handle.id())
    }
}

/// System that updates or initializes the [`spatial listener`](SpatialListener)'s position and orientation.
pub(crate) fn update_spatial_listener(
    ctx: &mut SystemsContext,
    mut query: Query<(&GlobalTransform, &mut SpatialListener), (With<Camera>, Changed<GlobalTransform>)>
) {
    let mut manager = ctx.resources.get_mut::<AudioManager>().unwrap();

    for (transform, listener) in query.iter_mut() {
        let position = transform.translation();
        let orientation = transform.rotation();

        if let Some(ref mut listener) = listener.0 {
            let instant_tween = Tween {
                duration: Duration::ZERO,
                ..Tween::default()
            };
            listener.set_position(position, instant_tween);
            listener.set_orientation(orientation, Tween::default());
        } else {
            listener.0 = Some(manager.add_listener(position, orientation).expect("Failed to add spatial listener"))
        }
    }
}
