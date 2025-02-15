mod manager;

pub use manager::*;

use crate::{assets::LoadableAsset, prelude::*};

use kira::sound::static_sound::StaticSoundData;

pub use kira;

/// Adds Audio playback functionality
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        let settings = AudioManagerSettings::default();
        let audio_manager = AudioManager::new(settings).expect("Failed to create AudioManager");

        app.set_resource(audio_manager);
        app.init_resource::<Assets<StaticSoundData>>();
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
