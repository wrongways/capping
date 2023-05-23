pub mod firestarter;
mod trial;
use std::fmt::{self, Display};
use crate::cli::CONFIGURATION;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CappingOrder {
    LevelBeforeActivate,
    LevelAfterActivate,
    LevelToLevel,
}

impl Display for CappingOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}",
            match self {
                Self::LevelBeforeActivate => "LevelBeforeActivate",
                Self::LevelAfterActivate => "LevelAfterActivate",
                Self::LevelToLevel => "LevelToLevel",
            }
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CappingOperation {
    Activate,
    Deactivate,
}

impl Display for CappingOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}",
            match self {
                Self::Activate => "Activate",
                Self::Deactivate => "Deactivate",
            }
        )
    }
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            cap_high_watts: CONFIGURATION.cap_high_watts,
            cap_low_watts: CONFIGURATION.cap_low_watts,
        }
    }

    /// Runs through the possible permutations of capping:
    /// * high power => low power
    /// * low power => high power
    /// * set cap level before activating capping
    /// * set cap level after activating capping
    pub fn run(&self) {
        for (cap_from, cap_to) in [
            (self.cap_low_watts, self.cap_high_watts),
            (self.cap_high_watts, self.cap_low_watts),
        ] {
            for capping_order in [
                CappingOrder::LevelBeforeActivate,
                CappingOrder::LevelAfterActivate,
                CappingOrder::LevelToLevel,
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
