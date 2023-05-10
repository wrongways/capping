pub mod firestarter;
mod trial;

use crate::cli::CONFIGURATION;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CappingOrder {
    LevelBeforeActivate,
    LevelAfterActivate,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CappingOperation {
    Activate,
    Deactivate,
}

pub struct Driver {
    cap_high_watts: u64,
    cap_low_watts: u64,
}

impl Default for Driver {
    fn default() -> Self {
        Self::new()
    }
}

impl Driver {
    pub fn new() -> Self {
        Self {
            cap_high_watts: CONFIGURATION.cap_high_watts,
            cap_low_watts: CONFIGURATION.cap_low_watts,
        }
    }

    pub fn run(&self) {
        for (cap_from, cap_to) in [
            (self.cap_low_watts, self.cap_high_watts),
            (self.cap_high_watts, self.cap_low_watts),
        ] {
            for capping_order in [
                CappingOrder::LevelBeforeActivate,
                CappingOrder::LevelAfterActivate,
            ] {
                for operation in [CappingOperation::Activate, CappingOperation::Deactivate] {
                    // No sense in running Level after Activate when operation is Deactivate
                    if !(capping_order == CappingOrder::LevelAfterActivate
                        && operation == CappingOperation::Deactivate)
                    {
                        let mut trial =
                            trial::Trial::new(cap_from, cap_to, capping_order, operation);
                        trial.run();
                    }
                }
            }
        }
    }
}
