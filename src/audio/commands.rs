use std::time::Duration;

use kira::{sound::Region, Tween};

use super::{AudioSource, Handle};

/// Commands for an [`audio track`](super::AudioTrack)
pub(crate) enum AudioCommand {
    Play(Handle<AudioSource>, Vec<AudioCommand>),
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

    pub(crate) fn tween_command(&mut self) -> TweenCommand {
        TweenCommand(self.tween_mut())
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
