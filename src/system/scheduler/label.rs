use std::{fmt::Debug, hash::Hash};

use crate::system::SchedulerLocation;

/// Marker trait for identifying system [phases](Phase).
/// To create a custom phase, implement this trait for a new type and register it with the
/// [`Scheduler`]
pub trait PhaseLabel: Debug + Clone + Copy + Send + Sync + Hash + 'static {
    /// Get the label for this phase
    #[inline]
    fn label() -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Set a layer within this phase
    #[inline]
    fn layer<L: LayerLabel>(&self, _layer: L) -> SchedulerLocation {
        SchedulerLocation {
            phase: Self::label(),
            layer: L::label(),
        }
    }
}

/// Marker trait for identifying system [layers](Layer).
/// To create a custom layer, implement this trait for a new type and register it with the
/// [`Scheduler`]
pub trait LayerLabel: Debug + Clone + Copy + Send + Sync + Hash + 'static {
    /// Get the label for this layer
    #[inline]
    fn label() -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub mod phase {
    macro_rules! create_phase_labels {
        ($($label:ident $doc:expr),*) => {
            $(
                #[doc=$doc]
                #[derive(Debug, Clone, Copy, Hash)]
                pub struct $label;
                impl super::PhaseLabel for $label {}
            )*

            /// Returns all built-in phase labels in order
            #[inline]
            pub fn all_phase_labels() -> Vec<&'static str> {
                vec![$(std::any::type_name::<$label>()),*]
            }
        };
    }

    create_phase_labels!(
        PreStartup "Runs before the [`Startup`] phase — used for very early initialization tasks such as logging or configuration loading.",
        Startup "Main startup phase, used for initializing game state, spawning entities, and loading assets.",
        First "Runs at the start of the frame update before other update logic.",
        PreUpdate "Runs immediately before [`Update`], useful for preparing frame data.",
        FixedUpdate "Runs at a fixed timestep, typically used for physics and time-step–dependent systems.",
        Update "Main per-frame update phase — most gameplay and logic systems run here.",
        PostUpdate "Runs immediately after [`Update`], typically used for state cleanup or deferred logic.",
        Last "Final update phase before rendering begins.",
        PreRender "Runs before the [`Render`] phase, often used to prepare render data.",
        Render "Main rendering phase, responsible for submitting GPU commands.",
        PostRender "Runs after the [`Render`] phase, often used for post-processing or readback tasks.",
        FrameEnd "Final phase of the frame; cleanup, diagnostics, and end-of-frame tasks go here."
    );
}

pub mod layer {
    macro_rules! create_layer_labels {
        ($($label:ident $doc:expr),*) => {
            $(
                #[doc=$doc]
                #[derive(Debug, Clone, Copy, Hash)]
                pub struct $label;
                impl super::LayerLabel for $label {}
            )*

            /// Returns all built-in layer labels
            #[inline]
            pub fn all_layer_labels() -> Vec<&'static str> {
                vec![$(std::any::type_name::<$label>()),*]
            }
        };
    }

    create_layer_labels!(
        Pre "Runs before all other layers in the phase. Used for early setup systems.",
        Start "Runs near the beginning of the phase. Useful for input or time updates.",
        Main "Primary layer in the phase — most game logic runs here.",
        End "Runs near the end of the phase, typically for cleanup or deferred execution.",
        Post "Runs after all other layers have completed. Useful for finalization tasks."
    );
}
