//! The application parses the command-line arguments (see below) creates the stats directory,
//! launches a monitor thread that drives BMC and RAPL monitors (see `monitor.rs`)
//! before launching the primary load driver (see `driver.rs`) which runs through the
//! test permutations.
//!
//! Once the load driver has completed, signal the monitor threads that they should exit
//! and save their results. The main threads waits for these child threads to complete
//! before exiting itself.

pub mod bmc;
pub mod cli;
pub mod driver;
pub mod monitor;
pub mod rapl;

use log::debug;
use sysconf::{self, SysconfVariable};

/// Generic result type - any error can be moved to `std::error::Error` type
pub type ResultType<T> = Result<T, Box<dyn std::error::Error>>;


///`core_count`
///
/// Use sysconf to read and return number of on-line cores
#[must_use]
pub fn core_count() -> u64 {
    #[allow(clippy::cast_sign_loss)]
    let cores = sysconf::sysconf(SysconfVariable::ScNprocessorsOnln)
        .expect("Couldn't get core count") as u64;
    debug!("Found {cores} online cores");
    cores
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_count() {
        let core_count = core_count();
        assert!(core_count > 0);
    }
}
