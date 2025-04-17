//! # Audio plugin
//! This plugin provides audio playback functionality using [`kira`](kira).
//! It allows you to play audio files, control playback, and manage audio tracks.
//!
//! ## Usage
//!
//! - Load audio files using the [`AssetLoader`]. It loads an [`AudioSource`].
//! ```ignore
//! let mut loader = ctx.resources.get_mut::<AssetLoader>().unwrap();
//! let source: AudioSource = loader.load("assets/sounds/loop.mp3", ctx.resources);
//! ```
//!
//! - To play the audio, get the [main track](AudioTrack) from the [`AudioManager`] and call `play` on it.
//! ```ignore
//! let mut audio = ctx.resources.get_mut::<AudioTrack>().unwrap();
//! audio.play().set_loop_region(0.0..); // Loop the whole sound
//! ```
//!
//! ## Spatial Audio
//!
//! - To play a sound spatially, add a [`SpatialEmitter`] component to an entity. You must have a
//! [`SpatialListener`] in the scene. The listener is usually on the camera.
//! ```ignore
//! let mut listener = SpatialListener::default(); // initializes in an update system
//! // ... add the listener to the camera entity
//!
//! let mut emitter = SpatialEmitter::default();
//! emitter.play(source);
//! // .. add the emitter to an entity
//! ```
//!
//! ## Audio Tracks
//!
//! - To create a new audio track, use the [`AudioManager`]. It's a wrapper around kira's
//! [manager](kira::AudioManager). These tracks are useful for organizing audio playback, they also
//! support tweening and other effects which are applied to the whole track.
//! ```ignore
//! let mut manager = ctx.resources.get_mut::<AudioManager>().unwrap();
//! let track = manager.add_sub_track(TrackBuilder::new()).unwrap();
//! let audio_track = AudioTrack::<YourTrackMarkerType>::new(track);
//! ```


mod manager;
mod track;
mod commands;
mod sound;
mod update;
mod spatial;

pub mod prelude {
    pub use super::AudioSource;
    pub use super::track::{AudioTrack, MainTrack};    
    pub use super::commands::{Easing, TweenCommand, PlayCommand};
    pub use super::sound::PlaybackState;
    pub use super::spatial::{SpatialListener, SpatialEmitter};
}

use std::{fmt::Debug, path::Path};

use crate::{assets::LoadableAsset, prelude::*};

// TODO: refactor audio once Added<C> and Removed<C> filters are implemented

use update::{cleanup_audio_tracks, update_audio_tracks, update_spatial_audio_tracks, update_spatial_listeners};
use kira::{sound::static_sound::StaticSoundData, track::TrackBuilder};
use manager::{AudioManager, AudioManagerSettings};

/// Source for an audio file, to play it use [`AudioTrack::play`]
///
/// To load an audio source use the [`AssetLoader`] like so:
/// ```ignore
/// let source = asset_loader.load::<AudioSource>("path/to/audio.ogg", resources);
/// ```
#[derive(Asset)]
pub struct AudioSource {
    source: StaticSoundData,
}

impl AudioSource {
    /// Creates a new audio source from [`kira`](kira)'s StaticSoundData
    pub fn new(source: StaticSoundData) -> Self {
        Self { source }
    }
}

/// Adds Audio playback functionality
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        let settings = AudioManagerSettings::default();
        let mut audio_manager = AudioManager::new(settings).expect("Failed to create AudioManager");

        let sub_track = audio_manager.add_sub_track(TrackBuilder::new()).expect("Failed to create main sub track");
        let main_track = AudioTrack::<MainTrack>::new(sub_track);

        app
            .set_resource(audio_manager)
            .set_resource(main_track)
            .init_resource::<Assets<AudioSource>>()

            // .add_system(update_spatial_listeners)
            // .add_system(update_audio_tracks)
            // .add_system(update_spatial_audio_tracks)
            // .add_system(cleanup_audio_tracks);

            // TODO: it has to be in Last stage since thats when GlobalTransform gets updated, once
            // Changed<C> works with a frame delay, it can be moved to the update stage. For now
            // there is no other way of change detection
            .register_system(update_spatial_listeners, SystemStage::Last)
            .register_system(update_audio_tracks, SystemStage::Last)
            .register_system(update_spatial_audio_tracks, SystemStage::Last)
            .register_system(cleanup_audio_tracks, SystemStage::Last);
    }
}

impl LoadableAsset for AudioSource {
    fn load<P: AsRef<Path> + Debug>(_: &mut AssetLoader, _: &mut Resources, path: P) -> Self {
        match StaticSoundData::from_file(path.as_ref()) {
            Ok(sound_data) => AudioSource::new(sound_data),
            Err(err) => panic!("Failed to load sound from '{:?}': {}", path, err),
        }   
    }
}
