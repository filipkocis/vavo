use std::{collections::{HashMap, VecDeque}, marker::PhantomData};

use kira::{sound::IntoOptionalRegion, track::{SpatialTrackHandle, TrackHandle}};

use crate::prelude::*;
use super::{commands::AudioCommand, sound::Sound, AudioSource, PlayCommand, TweenCommand};

/// Marker for the main [`audio track`](AudioTrack)
#[derive(Resource)]
pub struct MainTrack;

/// A spatial sub-track bound to an [`AudioTrack`]. Represents one [`SpatialEmitter`](crate::audio::spatial::SpatialEmitter)
#[derive(Resource)]
pub(crate) struct SpatialAudioTrack {
    pub(crate) sounds: Vec<Sound>,
    pub(crate) track: SpatialTrackHandle,
}

/// An audio track that can play multiple sounds, you can create multiple tracks. To use the
/// default [`main`](MainTrack) track use the [`AudioTrack`] resource
#[derive(Resource)]
pub struct AudioTrack<R: Resource = MainTrack> {
    pub(crate) commands: VecDeque<AudioCommand>,
    pub(crate) track: TrackHandle,
    pub(crate) sounds: Vec<Sound>,
    pub(crate) spatial_tracks: HashMap<EntityId, SpatialAudioTrack>,
    _marker: PhantomData<R>,
}

impl<R: Resource> AudioTrack<R> {
    pub fn new(track_handle: TrackHandle) -> Self {
        Self {
            commands: VecDeque::new(),
            track: track_handle,
            sounds: Vec::new(),
            spatial_tracks: HashMap::new(),
            _marker: PhantomData,
        }
    }

    /// Apply all queued commands
    pub(crate) fn apply(&mut self, resources: &mut Resources) {
        while let Some(command) = self.commands.pop_front() {
            match command {
                AudioCommand::Play(handle, commands) => {
                    let assets = resources.get::<Assets<AudioSource>>().expect("Assets<AudioSource> not found"); 
                    let sound_data = assets.get(&handle).expect("Failed to get sound data from assets");

                    let sound = match self.track.play(sound_data.source.clone()) {
                        Ok(sound) => sound,
                        Err(err) => panic!("Failed to play sound: {}", err),
                    };

                    let sound = Sound::new(sound, commands);
                    self.sounds.push(sound); 
                },

                AudioCommand::Pause(tween) => {
                    self.track.pause(tween);
                    self.spatial_tracks.values_mut().for_each(|track| track.track.pause(tween));
                },
                AudioCommand::Resume(tween) => {
                    self.track.resume(tween);
                    self.spatial_tracks.values_mut().for_each(|track| track.track.resume(tween));
                },
                AudioCommand::SetVolume(volume, tween) => {
                    self.track.set_volume(volume, tween);
                    self.spatial_tracks.values_mut().for_each(|track| track.track.set_volume(volume, tween));
                },

                command => self.sounds.iter_mut().for_each(|sound| sound.apply(command.clone()))
            }
        }
    }

    /// Pushes a command to the queue 
    fn push(&mut self, command: AudioCommand) -> &mut AudioCommand {
        self.commands.push_back(command);
        self.commands.back_mut().unwrap()
    }

    /// Plays an audio asset
    pub fn play(&mut self, source: Handle<AudioSource>) -> PlayCommand<'_> {
        self.push(AudioCommand::Play(source, Default::default())).play_command()
    }

    /// Stops all sounds
    pub fn stop(&mut self) -> TweenCommand<'_> {
        self.push(AudioCommand::Stop(Default::default())).tween_command()
    }

    /// Pauses all sounds
    pub fn pause(&mut self) -> TweenCommand<'_> {
        self.push(AudioCommand::Pause(Default::default())).tween_command()
    }

    /// Resumes all sounds
    pub fn resume(&mut self) -> TweenCommand<'_> {
        self.push(AudioCommand::Resume(Default::default())).tween_command()
    }

    /// Sets the volume of all sounds in decibels
    pub fn set_volume(&mut self, volume: f32) -> TweenCommand<'_> {
        self.push(AudioCommand::SetVolume(volume, Default::default())).tween_command()
    }

    /// Sets the panning of all sounds
    pub fn set_panning(&mut self, panning: f32) -> TweenCommand<'_> {
        self.push(AudioCommand::SetPanning(panning, Default::default())).tween_command()
    }

    /// Sets the playback rate of all sounds
    pub fn set_playback_rate(&mut self, rate: f64) -> TweenCommand<'_> {
        self.push(AudioCommand::SetPlaybackRate(rate, Default::default())).tween_command()
    }

    /// Sets the loop region of all sounds
    pub fn set_loop_region(&mut self, region: impl IntoOptionalRegion) {
        self.push(AudioCommand::SetLoopRegion(region.into_optional_region()));
    }
}

impl SpatialAudioTrack {
    pub fn new(track: SpatialTrackHandle) -> Self {
        Self {
            track,
            sounds: Vec::new(),
        }
    }

    /// Apply all queued commands to the spatial track
    pub(crate) fn apply(
        &mut self, 
        resources: &mut Resources, 
        commands: &mut VecDeque<AudioCommand>, 
    ) {
        while let Some(command) = commands.pop_front() {
            match command {
                AudioCommand::Play(handle, commands) => {
                    let assets = resources.get::<Assets<AudioSource>>().expect("Assets<AudioSource> not found"); 
                    let sound_data = assets.get(&handle).expect("Failed to get sound data from assets");

                    let sound = match self.track.play(sound_data.source.clone()) {
                        Ok(sound) => sound,
                        Err(err) => panic!("Failed to play sound: {}", err),
                    };

                    let sound = Sound::new(sound, commands);
                    self.sounds.push(sound); 
                },

                AudioCommand::Pause(tween) => self.track.pause(tween),
                AudioCommand::Resume(tween) => self.track.resume(tween),
                AudioCommand::SetVolume(volume, tween) => self.track.set_volume(volume, tween),

                command => self.sounds.iter_mut().for_each(|sound| sound.apply(command.clone()))
            }
        }
    }
}
