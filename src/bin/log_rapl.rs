use glob::glob;
use log::trace;
use simple_logger;
use std::fs::File;
use std::path::PathBuf;
use std::io::Read;
use chrono::{self, DateTime, Local};
use std::ffi::OsString;
use std::thread::sleep;
use std::time::Duration;

const RAPL_DIR: &str = "/sys/devices/virtual/powercap/intel-rapl/";
const RAPL_CORE_GLOB: &str = "intel-rapl:?/intel-rapl:?:0/energy_uj";

#[derive(Debug)]
#[allow(non_camel_case_types)]
struct RAPL_Data {
    pub timestamp: DateTime<Local>,
    pub domain: OsString,
    pub energy: usize,
}

impl RAPL_Data {
    fn new(domain: OsString, energy: usize) -> Self {
        Self {
            timestamp: Local::now(),
            domain,
            energy,
        }
    }
}

fn read_energy(filename: &PathBuf) -> usize {
    let mut s = String::new();
    let energy_string = File::open(filename)
        .expect("Failed to open RAPL file")
        .read_to_string(&mut s)
        .expect("Failed to read RAPL string");
    energy_string.into()
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let rapl_glob = format!("{RAPL_DIR}{RAPL_CORE_GLOB}");
    trace!("{rapl_glob}");
    let mut rapl_files = glob(&rapl_glob).expect("Failed to read rapl glob");
    let mut energy_readings = Vec::<RAPL_Data>::new();

    for _ in 0..7 {
        for rapl_file in &mut rapl_files {
            let f = &rapl_file.unwrap();
            #[allow(non_snake_case)]
            let domain = f.file_name().unwrap();
            energy_readings.push(RAPL_Data::new(domain.into(), read_energy(&f)));
        }
        sleep(Duration::from_secs(1));
    }

    for datapoint in energy_readings {
        trace!("{:?}", datapoint);
    }
}
