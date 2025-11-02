use crate::system::{ParamInfo, System};

/// A trait for checking conflicts between system parameters
pub(crate) trait ConflictChecker {
    /// Check if self is conflicting with other
    fn is_conflicting_with(&self, other: &Self) -> bool;
}

impl ConflictChecker for ParamInfo {
    fn is_conflicting_with(&self, other: &Self) -> bool {
        if self.type_info().type_id() != other.type_info().type_id() {
            return false;
        }

        self.is_mutable() || other.is_mutable()
    }
}

impl ConflictChecker for &[ParamInfo] {
    fn is_conflicting_with(&self, other: &Self) -> bool {
        for param_a in *self {
            for param_b in *other {
                if param_a.is_conflicting_with(param_b) {
                    return true;
                }
            }
        }
        false
    }
}

impl ConflictChecker for Vec<ParamInfo> {
    fn is_conflicting_with(&self, other: &Self) -> bool {
        self.as_slice().is_conflicting_with(&other.as_slice())
    }
}

impl ConflictChecker for System {
    fn is_conflicting_with(&self, other: &Self) -> bool {
        // self.Main vs Main
        if self
            .exec
            .params_info
            .is_conflicting_with(&other.exec.params_info)
        {
            return true;
        }

        // self.Main vs Condition
        for other_condition in &other.conditions {
            other_condition
                .exec
                .params_info
                .is_conflicting_with(&self.exec.params_info);
        }

        for condition in &self.conditions {
            // self.Condition vs Main
            if condition
                .exec
                .params_info
                .is_conflicting_with(&other.exec.params_info)
            {
                return true;
            }

            // self.Condition vs Condition
            for other_condition in &other.conditions {
                if condition
                    .exec
                    .params_info
                    .is_conflicting_with(&other_condition.exec.params_info)
                {
                    return true;
                }
            }
        }

        false
    }
}

impl ConflictChecker for &[System] {
    fn is_conflicting_with(&self, other: &Self) -> bool {
        for system_a in *self {
            for system_b in *other {
                if system_a.is_conflicting_with(system_b) {
                    return true;
                }
            }
        }
        false
    }
}

impl ConflictChecker for Vec<System> {
    fn is_conflicting_with(&self, other: &Self) -> bool {
        self.as_slice().is_conflicting_with(&other.as_slice())
    }
}
