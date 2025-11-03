mod changes;
pub mod label;
mod location;
mod phase;
mod threads;

pub use changes::SchedulerChanges;
pub use label::{LayerLabel, PhaseLabel};
pub use location::{IntoSchedulerLocation, SchedulerLocation};
pub use phase::{Phase, PhaseExecutionPolicy, PhaseExecutionType};
pub(super) use threads::ThreadPool;

use crate::{
    prelude::{FixedTime, World},
    system::{ConflictChecker, System},
};

/// A group of [systems](System) that can safely run in `parallel`.
///
/// Each batch contains systems that do not conflict with each other based on their
/// parameter access patterns. These systems can be `executed concurrently` for improved performance.
///
/// Batches are created automatically when inserting systems into [layers](Layer).
pub struct Batch {
    /// Systems in this batch
    systems: Vec<System>,
}

impl Batch {
    /// Create a new system batch
    #[inline]
    fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a system to this batch
    #[inline]
    fn add_system(&mut self, system: System) {
        self.systems.push(system);
    }

    /// Check if this batch can accept the given system without conflicts
    #[inline]
    fn can_accept(&self, system: &System) -> bool {
        for existing_system in &self.systems {
            if existing_system.is_conflicting_with(system) {
                return false;
            }
        }
        true
    }
}

/// A layer of systems within a [phase](Phase).
///
/// A layer provides an additional level of organization for systems, they can be `grouped` by
/// logical functionality such as `"Physics", "AI", or "Rendering"`. Layers also allow for
/// specifying `execution order` constraints between different groups of systems within the same
/// phase.
///
/// Layers always run sequentially, but systems within them can be [parallelized](Batch).
pub struct Layer {
    /// Layer label
    label: &'static str,
    /// Batches in this layer
    batches: Vec<Batch>,

    /// This layer will run before these layers
    before: Vec<&'static str>,
    /// This layer will run after these layers
    after: Vec<&'static str>,
}

impl Layer {
    /// Create a new layer
    #[inline]
    fn new(label: &'static str) -> Self {
        Self {
            label,
            batches: Vec::new(),
            before: Vec::new(),
            after: Vec::new(),
        }
    }

    /// Add a system to this layer
    #[inline]
    fn add_system(&mut self, system: System) {
        for batch in &mut self.batches {
            if batch.can_accept(&system) {
                batch.add_system(system);
                return;
            }
        }

        let mut new_batch = Batch::new();
        new_batch.add_system(system);
        self.batches.push(new_batch);
    }
}

/// The main scheduler responsible for organizing and executing the system pipeline.
///
/// The scheduler manages the overall structure of system execution by organizing systems into
/// [phases](Phase) and [layers](Layer). It ensures that systems are executed in the correct
/// topological order based on their specified dependencies.
///
/// The scheduler also handles the parallelization of systems within layers by grouping them into
/// [batches](Batch).
pub struct Scheduler {
    phases: Vec<Phase>,
    thread_pool: ThreadPool,
    pub pending_changes: SchedulerChanges,
}

impl Default for Scheduler {
    #[inline]
    fn default() -> Self {
        let phases = label::phase::all_phase_labels()
            .into_iter()
            .map(Phase::new)
            .collect();

        let size = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let mut scheduler = Self {
            phases,
            thread_pool: ThreadPool::new(size),
            pending_changes: SchedulerChanges::default(),
        };

        scheduler.pending_changes.policy(
            label::phase::FixedUpdate,
            PhaseExecutionPolicy::FixedTimestep(FixedTime::from_hz(60.0)),
        );
        scheduler
            .pending_changes
            .policy(label::phase::PreStartup, PhaseExecutionPolicy::Finite(1));
        scheduler
            .pending_changes
            .policy(label::phase::Startup, PhaseExecutionPolicy::Finite(1));

        scheduler.apply_changes();

        scheduler
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Print the current state of the scheduler for debugging
    pub fn debug_print(&self) {
        println!(
            "Scheduler Pending Changes: {}",
            self.pending_changes.changes.len()
        );

        println!("Scheduler Phases and Layers:");
        for phase in &self.phases {
            println!("  Phase: {:?}", phase.label);
            for layer in &phase.layers {
                println!("    Layer: {:?}", layer.label);
                for (batch_index, batch) in layer.batches.iter().enumerate() {
                    println!(
                        "      Batch {}: {} systems",
                        batch_index,
                        batch.systems.len()
                    );
                    for system in &batch.systems {
                        println!("        System: {:?}", system.exec.exec_info.type_name());
                    }
                }
            }
        }
    }

    /// Refresh the ordering of a mutated phase
    fn refresh_phase(&mut self, phase_label: &'static str) {
        if let Some(index) = self.find_phase(phase_label) {
            let phase = self.phases.remove(index);
            self.add_phase(phase);
        } else {
            panic!("System phase {:?} not found in scheduler", phase_label);
        }
    }

    /// Add a phase to the scheduler
    fn add_phase(&mut self, phase: Phase) {
        if self.get_phase_mut(phase.label).is_some() {
            panic!("System phase {:?} already exists in scheduler", phase.label);
        }

        let min_before = phase
            .before
            .iter()
            .map(|s| self.find_phase(s).unwrap_or(self.phases.len()))
            .min()
            .unwrap_or(self.phases.len());

        let max_after = phase.after.iter().filter_map(|s| self.find_phase(s)).max();

        let insert_index = match max_after {
            None => min_before,
            Some(max_after) if max_after < min_before => max_after + 1,
            Some(max_after) => {
                panic!(
                    "Conflicting phase ordering for phase {:?}, cannot insert before {:?} and after {:?} at the same time. Complete phase ordering: {:?}",
                    phase.label,
                    self.phases[min_before].label,
                    self.phases[max_after].label,
                    self.phases.iter().map(|x| x.label).collect::<Vec<_>>()
                )
            }
        };

        self.phases.insert(insert_index, phase);
    }

    /// Add a layer to a phase in the scheduler
    fn add_layer_to_phase(&mut self, phase_label: &'static str, layer: Layer) {
        if let Some(phase) = self.get_phase_mut(phase_label) {
            phase.add_layer(layer);
        } else {
            panic!("System phase {:?} not found in scheduler", phase_label);
        }
    }

    /// Get a mutable reference to a phase
    #[inline]
    fn get_phase_mut(&mut self, phase_label: &'static str) -> Option<&mut Phase> {
        self.phases.iter_mut().find(|s| s.label == phase_label)
    }

    /// Returns the phase index
    #[inline]
    fn find_phase(&self, phase_label: &'static str) -> Option<usize> {
        self.phases.iter().position(|s| s.label == phase_label)
    }

    /// Add a system to a specific location in the scheduler
    pub fn add_system(&mut self, system: System, location: impl IntoSchedulerLocation) {
        let phase_label = location.phase_label();
        let layer_label = location.layer_label();

        self.pending_changes
            .system_add(phase_label, layer_label, system);
    }

    /// Apply any pending changes to the scheduler
    #[inline]
    fn apply_changes(&mut self) {
        if self.pending_changes.changes.is_empty() {
            return;
        }

        let drained = self.pending_changes.changes.drain(..).collect::<Vec<_>>();
        for change in drained {
            change(self);
        }
    }
}

impl Scheduler {
    /// Execute the full scheduler pipeline
    #[inline]
    pub fn execute_pipeline(&mut self, world: &mut World) {
        self.apply_changes();

        for phase in &mut self.phases {
            phase.execute(world, &mut self.pending_changes, &self.thread_pool);
        }
    }

    /// Execute a specific phase in the scheduler
    #[inline]
    pub fn execute_phase(&mut self, world: &mut World, phase: impl PhaseLabel) {
        self.apply_changes();

        if let Some(phase_index) = self.find_phase(phase.phase_label()) {
            let phase = &mut self.phases[phase_index];
            phase.execute(world, &mut self.pending_changes, &self.thread_pool);
        } else {
            panic!(
                "System phase {:?} not found in scheduler",
                phase.phase_label()
            );
        }
    }
}
