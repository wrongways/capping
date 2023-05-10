use crate::cli::CONFIGURATION;
use crate::rapl::{RAPL_Readings, RAPL};
use crate::ResultType;
use chrono::{DateTime, Local};
use log::debug;
use log::{info, trace};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
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

fn save_rapl_stats(stats: &Vec<RAPL_Readings>) -> ResultType<()> {
    // Build the filename - append a timestamp and ".csv"
    let timestamp: DateTime<Local> = Local::now();

    // Have to format! because timestamp.format() produces a DelayedString, incompatible with Path
    let save_filename = format!("rapl_stats_{}.csv", timestamp.format("%y%m%d_%H%M"));
    let save_path = Path::new(&*CONFIGURATION.stats_dir).join(save_filename);
    debug!("Saving stats to: {}", save_path.to_str().unwrap());

    let handle = File::create(save_path)?;
    let mut writer = BufWriter::new(handle);

    // Write the header row

    let mut domains: String = stats[0]
        .readings
        .iter()
        .map(|&reading| String::from("domain_") + &reading.domain.to_string() + ",")
        .collect();
    // remove the extra comma
    domains.pop();
    writeln!(&mut writer, "timestamp,{domains}")?;

    // sanity check: ensure all reading have same # entries
    let n_domains = stats[0].readings.len();
    for stat in stats {
        assert_eq!(stat.readings.len(), n_domains);
        writeln!(&mut writer, "{stat}")?;
    }

    // file is automatically closed when it goes out of scope
    Ok(())
}
