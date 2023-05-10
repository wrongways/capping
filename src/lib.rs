pub mod bmc;
pub mod cli;
pub mod driver;
pub mod monitor;
pub mod rapl;

use crate::cli::CONFIGURATION;
use chrono::offset::Local;
use chrono::{DateTime, SecondsFormat};
use log::debug;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use sysconf::{self, SysconfVariable};

/// Generic result type - any error can be moved to std::error::Error type
pub type ResultType<T> = Result<T, Box<dyn std::error::Error>>;

/// Used for collecting power and energy stats with their timestamps
pub struct PowerStat {
    pub timestamp: DateTime<Local>,
    pub reading: u64,
}

impl PowerStat {
    pub fn new(reading: u64) -> Self {
        Self {
            timestamp: Local::now(),
            reading,
        }
    }
}

impl fmt::Display for PowerStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{}",
            self.timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
            self.reading
        )
    }
}

///`core_count`
///
/// Use sysconf to read and return number of on-line cores
pub fn core_count() -> u64 {
    #[allow(clippy::cast_sign_loss)]
    let cores = sysconf::sysconf(SysconfVariable::ScNprocessorsOnln)
        .expect("Couldn't get core count") as u64;
    debug!("Found {cores} online cores");
    cores
}

// TODO: Should provide an option to save to SQLite

/// `save_power_stats`
///
/// Builds a file path from the provided filename and the configuration stats_dir value
/// And appends a timestamp. Writes a vector of PowerStat entries to this file using a
/// buffered writer.
///
/// Returns the constructed path.
pub fn save_power_stats(filename: &str, stats: Vec<PowerStat>, col_name: &str) -> ResultType<PathBuf> {
    // Build the filename - append a timestamp and ".csv"
    let timestamp: DateTime<Local> = Local::now();

    // Have to format! because timestamp.format() produces a DelayedString, incompatible with Path
    let save_filename = format!("{filename}_{}.csv", timestamp.format("%y%m%d_%H%M"));
    let save_path = Path::new(&*CONFIGURATION.stats_dir).join(&save_filename);
    debug!(
        "Saving stats to: {}",
        save_path.to_str().expect("Failed to get save path")
    );

    let handle = File::create(&save_path)?;
    let mut writer = BufWriter::new(handle);

    // If a column name is provided, print a csv header
    if !col_name.is_empty() {
        writeln!(&mut writer, "timestamp,{col_name}")?;
    }

    for stat in stats {
        writeln!(&mut writer, "{stat}")?;
    }

    // file is automatically closed when it goes out of scope
    Ok(save_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_core_count() {
        let core_count = core_count();
        assert!(core_count > 0);
    }

    #[test]
    fn test_save_stats() {
        let s1 = PowerStat::new(42);
        let mut stats = Vec::<PowerStat>::new();
        stats.push(s1);
        fs::create_dir_all(&CONFIGURATION.stats_dir).expect("Failed to create stats directory");
        let s2 = PowerStat::new(0);
        stats.push(s2);
        let rc = save_power_stats("test_file", stats, "test_column");
        println!("{rc:?}");
        assert!(rc.is_ok());
        fs::remove_file(rc.unwrap()).expect("Test save stats, failed to remove stats file");
    }
}
