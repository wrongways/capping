use glob::glob;
use log::{trace, debug, error};
use simple_logger;
use chrono::{self, DateTime, Local};
use std::thread::sleep;
use std::time::Duration;
use std::str::FromStr;
use std::io::Write;
use std::path::Path;
use std::fs::{self, OpenOptions};


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

fn domain_from_path(path: &str) -> String {
    trace!("domain_from_path({})", path);
    let path = Path::new(path);
    let parent_dir = path.parent().expect("failed to get parent").file_name().expect("couldn't get parent filename");
    trace!("domain_from_path - parent dir: {:?}", parent_dir);
    let rapl_device: String = parent_dir.to_os_string().into_string().expect("Couldn't create string");
    let rapl_device_parts: Vec<&str> = rapl_device.split(':').collect();
    let domain = format!("{}:{}", rapl_device_parts[1], rapl_device_parts[2]);
    debug!("domain_from_path({}) -> {}", path.display(), domain);
    return domain;
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
            let rapl_domain = domain_from_path(rapl_file);
            debug!("Pushing new energy reading: {i}");
            energy_readings.push(RAPL_Data::new(rapl_domain, read_energy(rapl_file)));
        }
        sleep(Duration::from_secs(1));
    }

    trace!("Saving stats");
    write_stats(&energy_readings);
}
