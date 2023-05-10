use crate::cli::CONFIGURATION;
use crate::rapl::{RAPL_Readings, RAPL_Reading, RAPL};
use crate::ResultType;
use chrono::{self, DateTime, Local};
use log::debug;
use log::{info, trace};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

pub fn monitor_rapl(rx: &Receiver<()>) {
    info!("\tRAPL: launched");
    let runtime_estimate = (CONFIGURATION.warmup_secs + CONFIGURATION.test_time_secs) * 500;
    let mut stats = Vec::<RAPL_Readings>::with_capacity(runtime_estimate as usize);
    let rapl = RAPL::new();
    loop {
        if rx.try_recv().is_ok() {
            trace!("\tRAPL: got message - exiting");
            break;
        }
        stats.push(rapl.read_current_energy());
        trace!("\tRAPL: sleeping");
        thread::sleep(Duration::from_millis(1000));
    }
    save_rapl_stats(&stats).expect("Failed to save RAPL stats");
    info!("\tRAPL: Exiting");
}

fn save_rapl_stats(stats: &[RAPL_Readings]) -> ResultType<PathBuf> {
    // Build the filename - append a timestamp and ".csv"
    let timestamp: DateTime<Local> = Local::now();

    let save_filename = format!(
        "{}_{}.csv",
        CONFIGURATION.bmc_stats_filename_prefix,
        timestamp.format("%y%m%d_%H%M")
    );

    let save_path = Path::new(&CONFIGURATION.stats_dir).join(save_filename);
    debug!("Saving stats to: {}", save_path.to_str().unwrap());

    // Create buffered writer
    let handle = File::create(&save_path)?;
    let mut writer = BufWriter::new(handle);

    // Format the header row...
    let mut domains: String = stats[0]
        .readings
        .iter()
        .map(|&reading| String::from("domain_") + &reading.domain.to_string() + "_watts,")
        .collect();

    // remove the final extra comma
    domains.pop();

    // ... and write to file
    writeln!(&mut writer, "timestamp,{domains}")?;

    // Rather than recording the raw energy values, calculate the power for each domain
    for datapoint in convert_energy_to_power(&stats) {
        writeln!(&mut writer, "{datapoint}")?;
    }

    // file is automatically closed when it goes out of scope
    Ok(save_path)
}

fn convert_energy_to_power(stats: &[RAPL_Readings]) -> Vec<RAPL_Readings> {
    // The units of reading are µJ

    let mut readings = Vec::with_capacity(stats.len());
    // sanity check: ensure all reading have same # entries
    let n_domains = stats[0].readings.len();

    // for stats[1...], calculate power by calculating the
    // energy change from the previous reading and dividing by
    // the time delta for each RAPL domain. The total power is
    // the sum of the domains.

    // By using skip(1), the index from the enumerate is one behind the
    // current row, i.e. it points to the preceding row, which is exactly
    // what's needed to calculate the deltas.
    for (stat_index, stat) in stats.iter().skip(1).enumerate() {
        assert_eq!(stat.readings.len(), n_domains);
        let mut power_readings: Vec<RAPL_Reading> = Vec::with_capacity(n_domains);
        let time_delta = stat.timestamp - stats[stat_index].timestamp;
        let time_midpoint = stat.timestamp - (time_delta / 2);

        /*
         * This generates a lot of noise in the logs so only
         * uncomment if you're seeing problems with timestamps

        trace!(" Previous time: {}", stats[stat_index].timestamp);
        trace!("  Current time: {}", stat.timestamp);
        trace!("    Time delta: {:?}", time_delta);
        trace!("1/2 Time delta: {:?}", time_delta/2);
        trace!(" Time midpoint: {}\n", time_midpoint);

         */


        // Loop over the domains
        for (domain_index, reading) in stat.readings.iter().enumerate() {
            let energy_delta_uj = reading.reading - stats[stat_index].readings[domain_index].reading;
            let power_watts = energy_delta_uj / 1_000_000 / time_delta.num_seconds() as u64;
            power_readings.push(RAPL_Reading {
                domain: reading.domain,
                reading: power_watts,
            });
        }
        let datapoint = RAPL_Readings {timestamp: time_midpoint, readings: power_readings};
        readings.push(datapoint);
    }
    readings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_to_power() {
        let r1 = RAPL_Reading {domain: 0, reading: 0};
        let r2 = RAPL_Reading {domain: 1, reading: 0};
        let r3 = RAPL_Reading {domain: 0, reading: 100_000_000};
        let r4 = RAPL_Reading {domain: 1, reading:  50_000_000};
        let r5 = RAPL_Reading {domain: 0, reading: 200_000_000};
        let r6 = RAPL_Reading {domain: 1, reading: 100_000_000};
        let r7 = RAPL_Reading {domain: 0, reading: 200_000_000};
        let r8 = RAPL_Reading {domain: 1, reading: 100_000_000};
        let r9 = RAPL_Reading {domain: 0, reading: 400_000_000};
        let r10 = RAPL_Reading {domain: 1, reading: 200_000_000};

        let t0 = Local::now();
        let t1 = t0 + chrono::Duration::milliseconds(1000);
        let t2 = t0 + chrono::Duration::milliseconds(2000);
        let t3 = t0 + chrono::Duration::milliseconds(3000);
        let t4 = t0 + chrono::Duration::milliseconds(5000);

        let readings1 = RAPL_Readings{timestamp: t0, readings: vec![r1, r2]};
        let readings2 = RAPL_Readings{timestamp: t1, readings: vec![r3, r4]};
        let readings3 = RAPL_Readings{timestamp: t2, readings: vec![r5, r6]};
        let readings4 = RAPL_Readings{timestamp: t3, readings: vec![r7, r8]};
        let readings5 = RAPL_Readings{timestamp: t4, readings: vec![r9, r10]};

        let energy_stats = vec![readings1, readings2, readings3, readings4, readings5];
        let power_stats = convert_energy_to_power(&energy_stats);

        assert_eq!(power_stats.len(), energy_stats.len() - 1);

        // check power
        assert_eq!(power_stats[0].readings[0].reading, 100);
        assert_eq!(power_stats[0].readings[1].reading,  50);
        assert_eq!(power_stats[1].readings[0].reading, 100);
        assert_eq!(power_stats[1].readings[1].reading,  50);
        assert_eq!(power_stats[2].readings[0].reading,   0);
        assert_eq!(power_stats[2].readings[1].reading,   0);
        assert_eq!(power_stats[3].readings[0].reading, 100);
        assert_eq!(power_stats[3].readings[1].reading,  50);

        // check timestamps
        assert_eq!(power_stats[0].timestamp, t0 + chrono::Duration::milliseconds(500));
        assert_eq!(power_stats[1].timestamp, t0 + chrono::Duration::milliseconds(1500));
        assert_eq!(power_stats[2].timestamp, t0 + chrono::Duration::milliseconds(2500));
        assert_eq!(power_stats[3].timestamp, t0 + chrono::Duration::milliseconds(4000));
    }
}
