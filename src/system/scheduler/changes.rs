use crate::system::{
    IntoSchedulerLocation, Layer, LayerLabel, Phase, PhaseExecutionPolicy, PhaseExecutionType,
    PhaseLabel, Scheduler, System,
};

/// Function that applies a change to the [scheduler](Scheduler)
type SchedulerChange = Box<dyn FnOnce(&mut Scheduler) + Send + Sync>;

/// Changes to be applied to the [scheduler](Scheduler)
#[derive(Default)]
pub struct SchedulerChanges {
    pub(super) changes: Vec<SchedulerChange>,
}

impl SchedulerChanges {
    /// Internal scheduler change to remove a phase
    #[inline]
    pub(super) fn phase_remove(&mut self, phase_label: &'static str) -> &mut Self {
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
    pub(super) fn system_add(
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
