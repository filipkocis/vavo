use std::{fmt::Debug, hash::Hash};

use crate::{
    prelude::World,
    system::{SystemCondition, conflict::ConflictChecker},
};

use super::System;

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
    phase: &'static str,
    layer: &'static str,
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
    label: &'static str,
    /// Layers in this phase
    layers: Vec<Layer>,
    execution_type: PhaseExecutionType,
    execution_policy: PhaseExecutionPolicy,

    /// This phase will run before these phases
    before: Vec<&'static str>,
    /// This phase will run after these phases
    after: Vec<&'static str>,
}

/// Type of execution for a [phase](Phase)
#[derive(Default, Debug)]
pub enum PhaseExecutionType {
    /// Run systems sequentially (batches do not run in parallel)
    Sequential,
    /// Run systems in parallel (where possible)
    #[default]
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
    FixedTimestep(f64),
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
    fn get_fixed_timestep(&self) -> Option<f64> {
        match self {
            Self::FixedTimestep(timestep) => Some(*timestep),
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
            Self::FixedTimestep(timestep) => {
                write!(f, "PhaseExecutionPolicy::FixedTimestep({})", timestep)
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

impl Phase {
    /// Create a new phase
    #[inline]
    fn new(label: &'static str) -> Self {
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
    fn add_layer(&mut self, layer: Layer) {
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
    fn refresh_layer(&mut self, layer_label: &'static str) {
        if let Some(index) = self.find_layer(layer_label) {
            let layer = self.layers.remove(index);
            self.add_layer(layer);
        } else {
            panic!("System layer {:?} not found in scheduler", layer_label);
        }
    }

    /// Get a mutable reference to a layer
    #[inline]
    fn get_layer_mut(&mut self, layer_label: &'static str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|s| s.label == layer_label)
    }

    /// Returns the layer index
    #[inline]
    fn find_layer(&mut self, layer_label: &'static str) -> Option<usize> {
        self.layers.iter_mut().position(|s| s.label == layer_label)
    }

    /// Add a system to a specific layer in this phase
    fn add_system_to_layer(&mut self, layer_label: &'static str, system: System) {
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
    pub pending_changes: SchedulerChanges,
}

/// Function that applies a change to the [scheduler](Scheduler)
type SchedulerChange = Box<dyn FnOnce(&mut Scheduler) + Send + Sync>;
/// Changes to be applied to the [scheduler](Scheduler)
#[derive(Default)]
pub struct SchedulerChanges {
    changes: Vec<SchedulerChange>,
}

impl SchedulerChanges {
    /// Internal scheduler change to remove a phase
    #[inline]
    fn phase_remove(&mut self, phase_label: &'static str) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                if let Some(index) = scheduler.find_phase(phase_label) {
                    scheduler.phases.remove(index);
                } else {
                    panic!("System phase {:?} not found in scheduler", phase_label);
                }
            }));
        self
    }

    /// Internal scheduler change to add a system
    #[inline]
    fn system_add(
        &mut self,
        phase_label: &'static str,
        layer_label: &'static str,
        system: System,
    ) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                if let Some(phase) = scheduler.get_phase_mut(phase_label) {
                    phase.add_system_to_layer(layer_label, system);
                } else {
                    panic!("System phase {:?} not found in scheduler", phase_label);
                }
            }));
        self
    }

    /// Add a new phase to the scheduler
    pub fn phase_add<P: PhaseLabel>(&mut self, phase: P) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                scheduler.add_phase(Phase::new(phase.phase_label()));
            }));
        self
    }

    /// Add a new layer to the scheduler
    pub fn layer_add<L: IntoSchedulerLocation>(&mut self, location: L) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                scheduler
                    .add_layer_to_phase(location.phase_label(), Layer::new(location.layer_label()));
            }));
        self
    }

    /// Add a new before dependency to a phase
    pub fn phase_before<P: PhaseLabel, PB: PhaseLabel>(
        &mut self,
        phase: P,
        before: PB,
    ) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                let phase_label = phase.phase_label();

                let phase = scheduler
                    .get_phase_mut(phase_label)
                    .expect("Phase not found");
                phase.before.push(before.phase_label());

                scheduler.refresh_phase(phase_label);
            }));
        self
    }

    /// Add a new after dependency to a phase
    pub fn phase_after<P: PhaseLabel, PA: PhaseLabel>(&mut self, phase: P, after: PA) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                let phase_label = phase.phase_label();

                let phase = scheduler
                    .get_phase_mut(phase_label)
                    .expect("Phase not found");
                phase.after.push(after.phase_label());

                scheduler.refresh_phase(phase_label);
            }));
        self
    }

    /// Add a new before dependency to a layer
    pub fn layer_before<L: IntoSchedulerLocation, LB: LayerLabel>(
        &mut self,
        location: L,
        _before: LB,
    ) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                let phase_label = location.phase_label();
                let layer_label = location.layer_label();

                let phase = scheduler
                    .get_phase_mut(phase_label)
                    .expect("Phase not found");
                let layer = phase.get_layer_mut(layer_label).expect("Layer not found");
                layer.before.push(<LB as LayerLabel>::label());

                phase.refresh_layer(layer_label);
            }));
        self
    }

    /// Add a new after dependency to a layer
    pub fn layer_after<L: IntoSchedulerLocation, LA: LayerLabel>(
        &mut self,
        location: L,
        _after: LA,
    ) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                let phase_label = location.phase_label();
                let layer_label = location.layer_label();

                let phase = scheduler
                    .get_phase_mut(phase_label)
                    .expect("Phase not found");
                let layer = phase.get_layer_mut(layer_label).expect("Layer not found");
                layer.after.push(<LA as LayerLabel>::label());

                phase.refresh_layer(layer_label);
            }));
        self
    }

    /// Move a layer to a different phase
    pub fn move_layer<L: IntoSchedulerLocation, PT: PhaseLabel>(
        &mut self,
        location: L,
        target: PT,
    ) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                let phase_label = location.phase_label();
                let layer_label = location.layer_label();

                let target_phase_label = target.phase_label();

                let phase = scheduler
                    .get_phase_mut(phase_label)
                    .expect("Phase not found");
                let layer_index = phase.find_layer(layer_label).expect("Layer not found");
                let layer = phase.layers.remove(layer_index);

                let target_phase = scheduler
                    .get_phase_mut(target_phase_label)
                    .expect("Phase not found");
                target_phase.add_layer(layer);
            }));
        self
    }

    /// Set the execution policy for a phase
    pub fn policy<P: PhaseLabel>(&mut self, phase: P, policy: PhaseExecutionPolicy) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                let phase_label = phase.phase_label();

                let phase = scheduler
                    .get_phase_mut(phase_label)
                    .expect("Phase not found");
                phase.execution_policy = policy;
            }));
        self
    }

    /// Set the execution type for a phase
    pub fn set_type<P: PhaseLabel>(
        &mut self,
        phase: P,
        exec_type: PhaseExecutionType,
    ) -> &mut Self {
        self.changes
            .push(Box::new(move |scheduler: &mut Scheduler| {
                let phase_label = phase.phase_label();

                let phase = scheduler
                    .get_phase_mut(phase_label)
                    .expect("Phase not found");
                phase.execution_type = exec_type;
            }));
        self
    }
}

impl Default for Scheduler {
    #[inline]
    fn default() -> Self {
        let phases = phase::all_phase_labels()
            .into_iter()
            .map(Phase::new)
            .collect();

        let mut scheduler = Self {
            phases,
            pending_changes: SchedulerChanges::default(),
        };

        // scheduler.pending_changes.policy(
        //     phase::FixedUpdate,
        //     PhaseExecutionPolicy::FixedTimestep(1.0 / 60.0),
        // );
        scheduler
            .pending_changes
            .policy(phase::PreStartup, PhaseExecutionPolicy::Finite(1));
        scheduler
            .pending_changes
            .policy(phase::Startup, PhaseExecutionPolicy::Finite(1));

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
            phase.execute(world, &mut self.pending_changes);
        }
    }

    /// Execute a specific phase in the scheduler
    #[inline]
    pub fn execute_phase(&mut self, world: &mut World, phase: impl PhaseLabel) {
        self.apply_changes();

        if let Some(phase_index) = self.find_phase(phase.phase_label()) {
            let phase = &mut self.phases[phase_index];
            phase.execute(world, &mut self.pending_changes);
        } else {
            panic!(
                "System phase {:?} not found in scheduler",
                phase.phase_label()
            );
        }
    }
}

impl Phase {
    /// Execute this phase
    #[inline]
    fn execute(&mut self, world: &mut World, pending_changes: &mut SchedulerChanges) {
        if self.execution_policy.is_normal() {
            // Normal execution, run every frame
        } else if let Some(remaining) = self.execution_policy.decrement_finite() {
            if remaining == 0 {
                pending_changes.phase_remove(self.label);
            }
        } else if let Some(_timestep) = self.execution_policy.get_fixed_timestep() {
            todo!("Fixed timestep execution");
        } else if let Some(condition) = self.execution_policy.get_custom() {
            if !condition.run(world) {
                return;
            }
        } else {
            panic!("Unknown phase execution policy");
        }

        match self.execution_type {
            PhaseExecutionType::Sequential => self.execute_sequential(world),
            PhaseExecutionType::Parallel => self.execute_parallel(world),
        }

        // Apply system changes after execution on main thread
        self.apply_systems(world);

        // Flush any queued commands to the world
        world.flush_commands();
    }

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
