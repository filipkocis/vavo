use kira::{track::SpatialTrackBuilder, Tween};

use crate::prelude::*;

use super::{track::SpatialAudioTrack, AudioManager};

/// System that updates or initializes the [`spatial listener`](SpatialListener)'s position and orientation.
pub(crate) fn update_spatial_listeners(
    ctx: &mut SystemsContext,
    mut query: Query<(&GlobalTransform, &mut SpatialListener), (With<Camera>, Changed<GlobalTransform>)>
) {
    let mut manager = ctx.resources.get_mut::<AudioManager>();

    for (transform, listener) in query.iter_mut() {
        let position = transform.translation();
        let orientation = transform.rotation();

        if let Some(ref mut listener) = listener.0 {
            listener.set_position(position, Tween::default());
            listener.set_orientation(orientation, Tween::default());
        } else {
            listener.0 = Some(manager.add_listener(position, orientation).expect("Failed to add spatial listener"))
        }
    }
}

/// System which applies all queued audio track commands and updates the audio tracks
pub(crate) fn update_audio_tracks(ctx: &mut SystemsContext, _: Query<()>) {
    // TODO: currently only the main track is supported
    let mut audio = ctx.resources.get_mut::<AudioTrack>(); 
    audio.apply(ctx.resources);
}

/// System that updates or creates a spatial audio track for an [`emitter`](SpatialEmitter).
/// It also updates the spatial audio track's position.
///
/// If the emitter has all sounds stopped, the spatial track is removed from the
/// [`audio`](AudioTrack).
pub(crate) fn update_spatial_audio_tracks(ctx: &mut SystemsContext, mut query: Query<()>) {
    let listener_query = query.cast::<&SpatialListener, ()>().iter_mut();
    let Some(listener_id) = listener_query.get(0).map(|listener| listener.id()).flatten() else {
        // No listener found or listener not initialized
        return
    };

    // main audio track
    let mut audio = ctx.resources.get_mut::<AudioTrack>();

    // Update spatial track positions based on emitter's transform
    let mut moved_emitter_query = query.cast::<(EntityId, &GlobalTransform), (With<SpatialEmitter> ,Changed<GlobalTransform>)>();
    for (id, transform) in moved_emitter_query.iter_mut() {
        if let Some(spatial_track) = audio.spatial_tracks.get_mut(&id) {
            spatial_track.track.set_position(transform.translation(), Tween::default());
            continue
        } 
    }

    // Apply emitter commands, and create spatial tracks if they don't exist yet
    let mut emitter_commands_query = query.cast::<(EntityId, &SpatialEmitter, &GlobalTransform), Changed<SpatialEmitter>>();
    let mut mut_commands_query = query.cast::<&mut SpatialEmitter, ()>();
    for (id, emitter, transform) in emitter_commands_query.iter_mut() {
        // If commands are empty, do nothing. Commands can be empty if user didnt provide any or if
        // all sounds have finished and it got removed from the AudioTrack, so recreate it only if
        // it has commands.
        if emitter.commands.is_empty() {
            continue
        }

        // Request the same emitter again as &mut to avoid marking it as changed every frame
        let mut_emitter = mut_commands_query.get(id).expect("SpatialEmitter should exist");
        if let Some(spatial_track) = audio.spatial_tracks.get_mut(&id) {
            spatial_track.apply(ctx.resources, &mut mut_emitter.commands);
            continue
        }

        // Create spatial track
        let builder = SpatialTrackBuilder::new();
        let track_handle = audio.track.add_spatial_sub_track(listener_id, transform.translation(), builder)
            .expect("Failed to add spatial sub track");

        let mut spatial_track = SpatialAudioTrack::new(track_handle);
        spatial_track.apply(ctx.resources, &mut mut_emitter.commands);

        audio.spatial_tracks.insert(id, spatial_track);
    }
}

/// Removes all sounds that have stopped playing, and or all spatial audio tracks that have no
/// sounds playing.
pub(crate) fn cleanup_audio_tracks(ctx: &mut SystemsContext, mut check_emitter_query: Query<&SpatialEmitter>) {
    // TODO: currently only the main track is supported
    let mut audio = ctx.resources.get_mut::<AudioTrack>(); 

    // Remove stopped sounds from audio track
    audio.sounds.retain(|sound| !sound.is_stopped());

    // Remove spatial tracks with all sounds stopped
    audio.spatial_tracks.retain(|id, track| {
        // Remove spatial track if emitter component was removed, or entity despawned 
        if check_emitter_query.get(*id).is_none() {
            return false
        }

        track.sounds.retain(|sound| !sound.is_stopped());
        if track.sounds.is_empty() {
        }
        !track.sounds.is_empty()
    });
}
