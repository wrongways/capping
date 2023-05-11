use crate::bmc::{BMC, BMC_CapSetting};
use crate::cli::CONFIGURATION;
use crate::ResultType;
use log::{info, trace, debug};
use std::cmp::max;
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use chrono::{DateTime, Local};

#[allow(non_camel_case_types)]
#[derive(Debug)]
struct BMC_Stats {
    timestamp: DateTime<Local>,
    power: u64,
    cap_level: u64,
    cap_is_active: bool,
}

impl fmt::Display for BMC_Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{},{}",
            self.timestamp.to_rfc3339(),
            self.power,
            self.cap_level,
            self.cap_is_active,
        )
    }
}

impl BMC_Stats {
    pub fn new(power: u64, cap_settings: &BMC_CapSetting) -> Self {
        Self {
            timestamp: Local::now(),
            power,
            cap_level: cap_settings.power_limit,
            cap_is_active: cap_settings.is_active,
        }
    }
}

/// Periodically polls the BMC for power reading and saves the result. Runs on its own thread.
/// Each time through the loop, checks for a message from the main monitor thread that signals
/// that this thread can exit. Before exiting, saves results to CSV file.
pub fn monitor_bmc(rx: &Receiver<()>) {
    info!("\tBMC: launched");

    // An estimate of how long it takes to read the dcmi power values from the BMC
    const READ_LATENCY_EST_MS: u64 = 100;
    let thread_sleep_time_ms = max(0, (1000/CONFIGURATION.monitor_poll_freq_hz) - READ_LATENCY_EST_MS);

    let runtime_estimate = (CONFIGURATION.warmup_secs + CONFIGURATION.test_time_secs) * 500;
    let mut stats = Vec::<BMC_Stats>::with_capacity(runtime_estimate as usize);
    let bmc = BMC::new(
        &CONFIGURATION.bmc_hostname,
        &CONFIGURATION.bmc_username,
        &CONFIGURATION.bmc_password,
    );
    loop {
        // Check if monitor master asked us to exit with a message on the channel
        if rx.try_recv().is_ok() {
            trace!("\tBMC: got message - exiting");
            break;
        }

        // No message, read current power and capping status
        let current_power = bmc.current_power();
        let current_cap_settings = bmc.current_cap_settings();
        let reading = BMC_Stats::new(current_power, &current_cap_settings);

        trace!("BMC power reading: {reading:#?}");
        stats.push(reading);

        thread::sleep(Duration::from_millis(thread_sleep_time_ms));
    }

    save_bmc_stats(&stats).expect("Failed to save BMC stats");
    info!("\tBMC: Exiting");
}

/// `save_bmc_stats`
///
/// Builds a file path from configuration fields and appending
/// a timestamp. Writes a vector of `BMC_Stats` to this file using a
/// buffered writer.
///
/// Returns the constructed path in a Result<>.
fn save_bmc_stats(stats: &[BMC_Stats]) -> ResultType<PathBuf> {
    // Build a timestamped csv filename
    let timestamp_format = "%y%m%d_%H%M";
    let timestamp = Local::now().format(timestamp_format).to_string();
    let filename = format!("{}_{timestamp}.csv", CONFIGURATION.bmc_stats_filename_prefix);
    let filepath = Path::new(&CONFIGURATION.stats_dir).join(filename);
    debug!("Saving stats to: {filepath:?}");

    // Create buffered writer on the file
    let handle = File::create(&filepath)?;
    let mut writer = BufWriter::new(handle);

    // write csv header
    writeln!(writer, "timestamp,power,cap_limit,cap_is_active")?;

    // write the data
    for stat in stats {
        writeln!(writer, "{stat}")?;
    }
    Ok(filepath)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_save_bmc_stats() {
        let cap_settings1 = BMC_CapSetting {
            power_limit: 2000,
            is_active: false,
        };

        let cap_settings2 = BMC_CapSetting {
            power_limit: 600,
            is_active: true,
        };

        let s1 = BMC_Stats::new(1200, &cap_settings1);
        let mut stats = Vec::<BMC_Stats>::new();
        stats.push(s1);

        // main() is not run when running tests, so ensure
        // stats dir exists ourselves.
        fs::create_dir_all(&CONFIGURATION.stats_dir).expect("Failed to create stats directory");

        let s2 = BMC_Stats::new(500, &cap_settings2);
        stats.push(s2);
        let rc = save_bmc_stats(&stats);
        assert!(rc.is_ok());
        let stats_filepath = rc.unwrap();

        // todo: test file path & file content

        fs::remove_file(stats_filepath).expect("Test save stats, failed to remove stats file");
    }
}
