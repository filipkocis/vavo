use std::collections::VecDeque;

use kira::{listener::{ListenerHandle, ListenerId}, sound::IntoOptionalRegion};

use crate::prelude::*;

use super::commands::AudioCommand;

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

/// Component which makes an entity a spatial audio emitter. You can play as many sounds as you
/// want. They are attached to the main [`AudioTrack`].
///
/// Despawning the entity or removing the component will stop all sounds.
#[derive(Component, Default, Debug)]
pub struct SpatialEmitter {
    pub(crate) commands: VecDeque<AudioCommand>,
    // pub(crate) track: Option<SpatialTrackHandle>,
}

impl SpatialEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a command to the queue 
    fn push(&mut self, command: AudioCommand) -> &mut AudioCommand {
        self.commands.push_back(command);
        self.commands.back_mut().unwrap()
    }

    /// Spatially plays an audio asset
    pub fn play(&mut self, source: Handle<AudioSource>) -> PlayCommand {
        self.push(AudioCommand::Play(source, Default::default())).play_command()
    }

    /// Stops all spatial sounds
    pub fn stop(&mut self) -> TweenCommand {
        self.push(AudioCommand::Stop(Default::default())).tween_command()
    }

    /// Pauses all spatial sounds
    pub fn pause(&mut self) -> TweenCommand {
        self.push(AudioCommand::Pause(Default::default())).tween_command()
    }

    /// Resumes all spatial sounds
    pub fn resume(&mut self) -> TweenCommand {
        self.push(AudioCommand::Resume(Default::default())).tween_command()
    }

    /// Sets the volume of all spatial sounds in decibels
    pub fn set_volume(&mut self, volume: f32) -> TweenCommand {
        self.push(AudioCommand::SetVolume(volume, Default::default())).tween_command()
    }

    /// Sets the panning of all spatial sounds
    pub fn set_panning(&mut self, panning: f32) -> TweenCommand {
        self.push(AudioCommand::SetPanning(panning, Default::default())).tween_command()
    }

    /// Sets the playback rate of all spatial sounds
    pub fn set_playback_rate(&mut self, rate: f64) -> TweenCommand {
        self.push(AudioCommand::SetPlaybackRate(rate, Default::default())).tween_command()
    }

    /// Sets the loop region of all spatial sounds
    pub fn set_loop_region(&mut self, region: impl IntoOptionalRegion) {
        self.push(AudioCommand::SetLoopRegion(region.into_optional_region()));
    }
}
