use glob::glob;
use log::{trace, debug};
use simple_logger;
use std::fs;
use std::path::PathBuf;
use chrono::{self, DateTime, Local};
use std::ffi::OsString;
use std::thread::sleep;
use std::time::Duration;
use std::str::FromStr;

const RAPL_DIR: &str = "/sys/devices/virtual/powercap/intel-rapl/";
const RAPL_CORE_GLOB: &str = "intel-rapl:?/intel-rapl:?:0/energy_uj";

#[derive(Debug)]
#[allow(non_camel_case_types)]
struct RAPL_Data {
    pub timestamp: DateTime<Local>,
    pub domain: OsString,
    pub energy: u64,
}

impl RAPL_Data {
    fn new(domain: OsString, energy: u64) -> Self {
        Self {
            timestamp: Local::now(),
            domain,
            energy,
        }
    }
}

fn read_energy(filename: &PathBuf) -> u64 {
    let energy_reading: String = fs::read_to_string(filename).unwrap();
    trace!("read_energy({:?} => {}", filename, energy_reading);
    let energy_reading: u64 = u64::from_str(energy_reading.trim()).unwrap();
    trace!("read_energy({:?} => {}", filename, energy_reading);
    energy_reading
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let rapl_glob = format!("{RAPL_DIR}{RAPL_CORE_GLOB}");
    let mut rapl_files = glob(&rapl_glob).expect("Failed to read rapl glob");
    let mut energy_readings = Vec::<RAPL_Data>::new();

    for i in 0..5 {
        for rapl_file in &mut rapl_files {
            let f = &rapl_file.unwrap();
            #[allow(non_snake_case)]
            let domain = f.parent().unwrap();
            debug!("Pushing new energy reading");
            energy_readings.push(RAPL_Data::new(domain.into(), read_energy(&f)));
        }
        sleep(Duration::from_secs(1));
    }

    for datapoint in energy_readings {
        trace!("Datapoint: {:?}", &datapoint);
    }
}