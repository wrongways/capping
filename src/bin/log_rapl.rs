use glob::glob;
use log::{trace, debug, error};
use simple_logger;
use std::fs::{self, OpenOptions};
use chrono::{self, DateTime, Local};
use std::thread::sleep;
use std::time::Duration;
use std::str::FromStr;
use std::string::ToString;
use std::io::Write;

const RAPL_DIR: &str = "/sys/devices/virtual/powercap/intel-rapl/";
const RAPL_CORE_GLOB: &str = "intel-rapl:?/intel-rapl:?:0/energy_uj";
const STATS_FILE: &str = "rapl_stats.csv";

#[derive(Debug)]
#[allow(non_camel_case_types)]
struct RAPL_Data {
    pub timestamp: DateTime<Local>,
    pub domain: String,
    pub energy: u64,
}

impl RAPL_Data {
    fn new(domain: String, energy: u64) -> Self {
        Self {
            timestamp: Local::now(),
            domain,
            energy,
        }
    }
}

fn read_energy(filename: &str) -> u64 {
    let energy_reading: String = fs::read_to_string(filename).unwrap();
    trace!("read_energy string({:?} => {}", filename, energy_reading);
    let energy_reading: u64 = u64::from_str(energy_reading.trim()).unwrap();
    trace!("read_energy u64({:?} => {}", filename, energy_reading);
    energy_reading
}

fn write_stats(stats: &Vec<RAPL_Data>) {
    let mut outfile = OpenOptions::new()
        .append(true)
        .create(true)
        .open(STATS_FILE)
        .unwrap();

    for stat in stats {
        if let Err(why) = writeln!(&mut outfile, "{},{},{}", stat.timestamp, stat.domain, stat.energy) {
            error!("Couldn't write to file {}: {}", STATS_FILE, why);
        }
    }
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let rapl_glob = format!("{RAPL_DIR}{RAPL_CORE_GLOB}");
    let rapl_paths = glob(&rapl_glob).expect("Failed to read rapl glob");
    let mut rapl_files = Vec::<String>::new();
    for p in rapl_paths {
        match p {
            Ok(path) => {
                let path = path.into_os_string().into_string().unwrap();
                rapl_files.push(path);
            },
            Err(why) => debug!("Titsup on path: {why:?}"),
        }
    }
    let mut energy_readings = Vec::<RAPL_Data>::new();

    for i in 0..5 {
        for rapl_file in &rapl_files {
            debug!("Pushing new energy reading: {i}");
            energy_readings.push(RAPL_Data::new(rapl_file.to_string(), read_energy(rapl_file)));
        }
        sleep(Duration::from_secs(1));
    }

    trace!("Saving stats");
    write_stats(&energy_readings);
}
