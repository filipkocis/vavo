use std::ops::{Deref, DerefMut};

use crate::prelude::*;

use kira::{backend::Backend, DefaultBackend};

/// World resource that controls the audio
#[derive(Resource)]
pub(crate) struct AudioManager(kira::AudioManager);

/// Settings for [`AudioManager`]
#[derive(Default)]
pub(crate) struct AudioManagerSettings(kira::AudioManagerSettings<DefaultBackend>);

impl AudioManager {
    pub fn new(settings: AudioManagerSettings) -> Result<Self, <DefaultBackend as Backend>::Error> {
        kira::AudioManager::<DefaultBackend>::new(settings.0).map(Self)
    }
}

impl Deref for AudioManager {
    type Target = kira::AudioManager;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AudioManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
