mod manager;
mod track;
mod commands;
mod sound;
mod apply;

pub mod prelude {
    pub use super::track::{AudioTrack, MainTrack};    
    pub use super::commands::{Easing, TweenCommand, PlayCommand};
    pub use super::sound::PlaybackState;
}

use crate::{assets::LoadableAsset, prelude::*};

use apply::apply_audio_track_commands;
use kira::{sound::static_sound::StaticSoundData, track::TrackBuilder};
use manager::{AudioManager, AudioManagerSettings};

/// Source for an audio file, to play it use [`AudioTrack::play`]
#[derive(Asset)]
pub struct AudioSource {
    source: StaticSoundData,
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
            .add_system(apply_audio_track_commands);
    }
}

impl Asset for StaticSoundData {}

impl LoadableAsset for StaticSoundData {
    fn load(_: &mut AssetLoader, _: &mut Resources, path: &str) -> Self {
        match StaticSoundData::from_file(path) {
            Ok(sound_data) => sound_data,
            Err(err) => panic!("Failed to load sound from '{}': {}", path, err),
        }   
    }
}
