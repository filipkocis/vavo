use std::collections::VecDeque;

use kira::sound::static_sound::StaticSoundHandle;

use super::commands::AudioCommand;

/// A sound which may or may not be currently playing
pub(crate) struct Sound(pub(crate) StaticSoundHandle);

pub type PlaybackState = kira::sound::PlaybackState;

impl Sound {
    pub fn new(handle: StaticSoundHandle, commands: VecDeque<AudioCommand>) -> Self {
        let mut sound = Self(handle);
        commands.into_iter().for_each(|command| sound.apply(command));
        sound
    }

    /// Returns the current playback state of the sound
    pub fn state(&self) -> PlaybackState {
        self.0.state()
    }

    /// Apply a command to the sound
    pub(crate) fn apply(&mut self, command: AudioCommand) {
        match command {
            AudioCommand::Play(..) => panic!("Play command is not valid for a sound"),
            AudioCommand::Pause(tween) => self.0.pause(tween),
            AudioCommand::Resume(tween) => self.0.resume(tween),
            AudioCommand::Stop(tween) => self.0.stop(tween),
            AudioCommand::SetVolume(volume, tween) => self.0.set_volume(volume, tween),
            AudioCommand::SetPanning(panning, tween) => self.0.set_panning(panning, tween),
            AudioCommand::SetPlaybackRate(rate, tween) => self.0.set_playback_rate(rate, tween),
            AudioCommand::SetLoopRegion(region) => self.0.set_loop_region(region),
        }
    }
}
