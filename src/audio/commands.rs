use std::{collections::VecDeque, time::Duration};

use kira::{sound::{IntoOptionalRegion, Region}, Tween};

use super::{AudioSource, Handle};

/// Commands for an [`audio track`](super::AudioTrack)
#[derive(Debug, Clone)]
pub(crate) enum AudioCommand {
    Play(Handle<AudioSource>, VecDeque<AudioCommand>),
    Pause(Tween),
    Resume(Tween),
    Stop(Tween),
    SetVolume(f32, Tween),
    SetPanning(f32, Tween),
    SetPlaybackRate(f64, Tween),
    SetLoopRegion(Option<Region>),
}

impl AudioCommand {
    pub(crate) fn tween(&self) -> &Tween {
        match self {
            Self::Play(..) => panic!("Play command does not have a tween"),
            Self::Pause(tween) => tween,
            Self::Resume(tween) => tween,
            Self::Stop(tween) => tween,
            Self::SetVolume(_, tween) => tween,
            Self::SetPanning(_, tween) => tween,
            Self::SetPlaybackRate(_, tween) => tween,
            Self::SetLoopRegion(_) => panic!("Loop region command does not have a tween"),
        }
    }

    pub(crate) fn tween_mut(&mut self) -> &mut Tween {
        match self {
            Self::Play(..) => panic!("Play command does not have a tween"),
            Self::Pause(tween) => tween,
            Self::Resume(tween) => tween,
            Self::Stop(tween) => tween,
            Self::SetVolume(_, tween) => tween,
            Self::SetPanning(_, tween) => tween,
            Self::SetPlaybackRate(_, tween) => tween,
            Self::SetLoopRegion(_) => panic!("Loop region command does not have a tween"),
        }
    }

    /// Returns the tween command or panics
    pub(crate) fn tween_command(&mut self) -> TweenCommand {
        TweenCommand(self.tween_mut())
    }

    /// Returns the play command for [`Self::Play`] or panics
    pub(crate) fn play_command(&mut self) -> PlayCommand {
        match self {
            Self::Play(_, commands) => PlayCommand(commands),
            _ => panic!("Expected a play command"),
        }
    }
}

pub type Easing = kira::Easing;

/// Commands to configure a [`Tween`] for an [`AudioCommand`]
pub struct TweenCommand<'a>(&'a mut Tween);

impl TweenCommand<'_> {
    /// Sets the delay before the tween starts, `0` means immediate start
    pub fn set_delay(&mut self, delay: Duration) -> &mut Self {
        if delay.is_zero() {
            self.0.start_time = kira::StartTime::Immediate;
        } else {
            self.0.start_time = kira::StartTime::Delayed(delay);
        }

        self
    }

    /// Sets the duration of the tween
    pub fn set_duration(&mut self, duration: Duration) -> &mut Self {
        self.0.duration = duration;
        self
    }

    /// Sets the easing function of the tween
    pub fn set_easing(&mut self, easing: Easing) -> &mut Self{
        self.0.easing = easing;
        self
    }
}

/// Commands for a new [`sound`](super::sound::Sound) to play
pub struct PlayCommand<'a>(&'a mut VecDeque<AudioCommand>);

impl PlayCommand<'_> {
    /// Pushes a command to the queue 
    fn push(&mut self, command: AudioCommand) -> &mut AudioCommand {
        self.0.push_back(command);
        self.0.back_mut().unwrap()
    }

    /// Stops this sound
    pub fn stop(&mut self) -> TweenCommand {
        self.push(AudioCommand::Stop(Default::default())).tween_command()
    }

    /// Pauses this sound
    pub fn pause(&mut self) -> TweenCommand {
        self.push(AudioCommand::Pause(Default::default())).tween_command()
    }

    /// Resumes this sound
    pub fn resume(&mut self) -> TweenCommand {
        self.push(AudioCommand::Resume(Default::default())).tween_command()
    }

    /// Sets the volume in decibels
    pub fn set_volume(&mut self, volume: f32) -> TweenCommand {
        self.push(AudioCommand::SetVolume(volume, Default::default())).tween_command()
    }

    /// Sets the panning
    pub fn set_panning(&mut self, panning: f32) -> TweenCommand {
        self.push(AudioCommand::SetPanning(panning, Default::default())).tween_command()
    }

    /// Sets the playback rate
    pub fn set_playback_rate(&mut self, rate: f64) -> TweenCommand {
        self.push(AudioCommand::SetPlaybackRate(rate, Default::default())).tween_command()
    }

    /// Sets the loop region
    pub fn set_loop_region(&mut self, region: impl IntoOptionalRegion) {
        self.push(AudioCommand::SetLoopRegion(region.into_optional_region()));
    }
}
