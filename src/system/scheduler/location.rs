use std::fmt::Debug;

use crate::system::{LayerLabel, PhaseLabel, layer, phase};

/// Implemented for types that can specify a location of a system within the
/// [scheduler](Scheduler).
pub trait IntoSchedulerLocation: Debug + Send + Sync + 'static {
    /// Get the phase label for this location
    #[inline]
    fn phase_label(&self) -> &'static str {
        phase::Update::label()
    }

    /// Get the layer label for this location
    #[inline]
    fn layer_label(&self) -> &'static str {
        layer::Main::label()
    }

    /// Get the full scheduler location
    #[inline]
    fn get(&self) -> SchedulerLocation {
        SchedulerLocation {
            phase: self.phase_label(),
            layer: self.layer_label(),
        }
    }
}

/// A scheduler location of a system
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SchedulerLocation {
    pub(super) phase: &'static str,
    pub(super) layer: &'static str,
}

impl Default for SchedulerLocation {
    #[inline]
    fn default() -> Self {
        Self {
            phase: phase::Update::label(),
            layer: layer::Main::label(),
        }
    }
}

impl IntoSchedulerLocation for SchedulerLocation {
    #[inline]
    fn phase_label(&self) -> &'static str {
        self.phase
    }

    #[inline]
    fn layer_label(&self) -> &'static str {
        self.layer
    }
}

impl<P: PhaseLabel> IntoSchedulerLocation for P {
    #[inline]
    fn phase_label(&self) -> &'static str {
        P::label()
    }
}
