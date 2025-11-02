use std::fmt::Debug;

use crate::{
    prelude::{FixedTime, World},
    system::{Layer, SchedulerChanges, System, SystemCondition, layer},
};

/// Type of execution for a [phase](Phase)
#[derive(Default, Debug)]
pub enum PhaseExecutionType {
    /// Run systems sequentially (batches do not run in parallel)
    #[default]
    Sequential,
    /// Run systems in parallel (where possible)
    Parallel,
}

/// Policy for executing a [phase](Phase)
#[derive(Default)]
pub enum PhaseExecutionPolicy {
    /// Default execution policy, runs systems every frame
    #[default]
    Normal,
    /// Run systems for a finite number of iterations, then they are removed from the scheduler
    Finite(usize),
    /// Run systems at a fixed timestap
    FixedTimestep(FixedTime),
    /// Run systems based on a custom condition
    Custom(SystemCondition),
}

impl PhaseExecutionPolicy {
    #[inline]
    fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    #[inline]
    fn decrement_finite(&mut self) -> Option<usize> {
        match self {
            Self::Finite(iterations) if *iterations > 0 => {
                *iterations -= 1;
                Some(*iterations)
            }
            _ => None,
        }
    }

    #[inline]
    fn get_fixed_timestep(&mut self) -> Option<&mut FixedTime> {
        match self {
            Self::FixedTimestep(timestep) => Some(timestep),
            _ => None,
        }
    }

    #[inline]
    fn get_custom(&mut self) -> Option<&mut SystemCondition> {
        match self {
            Self::Custom(condition) => Some(condition),
            _ => None,
        }
    }
}

impl Debug for PhaseExecutionPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "PhaseExecutionPolicy::Normal"),
            Self::Finite(iterations) => write!(f, "PhaseExecutionPolicy::Finite({})", iterations),
            Self::FixedTimestep(time) => {
                write!(
                    f,
                    "PhaseExecutionPolicy::FixedTimestep({})",
                    time.fixed_delta()
                )
            }
            Self::Custom(condition) => {
                write!(
                    f,
                    "PhaseExecutionPolicy::Custom({:?})",
                    condition.exec.exec_info.type_name()
                )
            }
        }
    }
}

/// A phase of system execution within the [scheduler](Scheduler) pipeline.
///
/// Phases provide a high-level organization for systems, they create logical groupings such as
/// `"Startup", "Update", or "Render"`. They are similar to [layers](Layer) in that they allow for
/// specifying `execution order` constraints and grouping systems, but phases operate at a higher
/// level.
///
/// You should prefer using phases to separate major stages of your application's lifecycle,
/// while layers are better suited for organizing related systems within those stages.
///
/// Phases always run sequentially.
pub struct Phase {
    /// Phase label
    pub(super) label: &'static str,
    /// Layers in this phase
    pub(super) layers: Vec<Layer>,
    pub(super) execution_type: PhaseExecutionType,
    pub(super) execution_policy: PhaseExecutionPolicy,

    /// This phase will run before these phases
    pub(super) before: Vec<&'static str>,
    /// This phase will run after these phases
    pub(super) after: Vec<&'static str>,
}

impl Phase {
    /// Create a new phase
    #[inline]
    pub(super) fn new(label: &'static str) -> Self {
        let layers = layer::all_layer_labels()
            .into_iter()
            .map(Layer::new)
            .collect();

        Self {
            label,
            layers,
            execution_type: PhaseExecutionType::default(),
            execution_policy: PhaseExecutionPolicy::default(),
            before: Vec::new(),
            after: Vec::new(),
        }
    }

    /// Add a layer to this phase
    pub(super) fn add_layer(&mut self, layer: Layer) {
        if self.get_layer_mut(layer.label).is_some() {
            panic!(
                "System layer {:?} already exists in phase {:?}",
                layer.label, self.label
            );
        }

        let min_before = layer
            .before
            .iter()
            .map(|s| self.find_layer(s).unwrap_or(self.layers.len()))
            .min()
            .unwrap_or(self.layers.len());

        let max_after = layer.after.iter().filter_map(|s| self.find_layer(s)).max();

        let insert_index = match max_after {
            None => min_before,
            Some(max_after) if max_after < min_before => max_after + 1,
            Some(max_after) => {
                panic!(
                    "Conflicting layer ordering for layer {:?}, cannot insert before {:?} and after {:?} at the same time. Complete layer ordering: {:?}",
                    layer.label,
                    self.layers[min_before].label,
                    self.layers[max_after].label,
                    self.layers.iter().map(|x| x.label).collect::<Vec<_>>()
                )
            }
        };

        self.layers.insert(insert_index, layer);
    }

    /// Refresh the ordering of a mutated layer
    pub(super) fn refresh_layer(&mut self, layer_label: &'static str) {
        if let Some(index) = self.find_layer(layer_label) {
            let layer = self.layers.remove(index);
            self.add_layer(layer);
        } else {
            panic!("System layer {:?} not found in scheduler", layer_label);
        }
    }

    /// Get a mutable reference to a layer
    #[inline]
    pub(super) fn get_layer_mut(&mut self, layer_label: &'static str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|s| s.label == layer_label)
    }

    /// Returns the layer index
    #[inline]
    pub(super) fn find_layer(&mut self, layer_label: &'static str) -> Option<usize> {
        self.layers.iter_mut().position(|s| s.label == layer_label)
    }

    /// Add a system to a specific layer in this phase
    pub(super) fn add_system_to_layer(&mut self, layer_label: &'static str, system: System) {
        if let Some(layer) = self.get_layer_mut(layer_label) {
            layer.add_system(system);
        } else {
            panic!(
                "System layer {:?} not found in phase {:?}",
                layer_label, self.label
            );
        }
    }
}

impl Phase {
    /// Execute this phase
    #[inline]
    pub(super) fn execute(&mut self, world: &mut World, pending_changes: &mut SchedulerChanges) {
        let mut iterations = 1;

        if self.execution_policy.is_normal() {
            // Normal execution, run every frame
        } else if let Some(remaining) = self.execution_policy.decrement_finite() {
            if remaining == 0 {
                pending_changes.phase_remove(self.label);
            }
        } else if let Some(timestep) = self.execution_policy.get_fixed_timestep() {
            timestep.update();
            iterations = timestep.iter();
        } else if let Some(condition) = self.execution_policy.get_custom() {
            if !condition.run(world) {
                return;
            }
        } else {
            panic!("Unknown phase execution policy");
        }

        // Execute systems for the determined number of iterations
        for _ in 0..iterations {
            match self.execution_type {
                PhaseExecutionType::Sequential => self.execute_sequential(world),
                PhaseExecutionType::Parallel => self.execute_parallel(world),
            }
        }

        // Apply system changes after execution on main thread
        self.apply_systems(world);

        // Flush any queued commands to the world
        world.flush_commands();
    }

    /// Execute systems in this phase sequentially
    #[inline]
    fn execute_sequential(&mut self, world: &mut World) {
        for layer in &mut self.layers {
            for batch in &mut layer.batches {
                for system in &mut batch.systems {
                    system.run(world);
                }
            }
        }
    }

    /// Execute systems in parallel where possible
    /// TODO: Currently temporary solution using new threads, replace with proper thread pool
    #[inline]
    fn execute_parallel(&mut self, world: &mut World) {
        let mut handles = vec![];
        for layer in &mut self.layers {
            for batch in &mut layer.batches {
                for system in &mut batch.systems {
                    let world_ref = unsafe { &mut *(world as *mut World) };
                    let system_ref = unsafe { &mut *(system as *mut System) };

                    let handle = std::thread::spawn(move || {
                        system_ref.run(world_ref);
                    });

                    handles.push(handle);
                }

                for handle in handles.drain(..) {
                    handle.join().expect("System thread panicked");
                }
            }
        }
    }

    /// Apply all systems
    #[inline]
    fn apply_systems(&mut self, world: &mut World) {
        for layer in &mut self.layers {
            for batch in &mut layer.batches {
                for system in &mut batch.systems {
                    system.apply(world);
                }
            }
        }
    }
}
