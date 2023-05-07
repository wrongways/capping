use glob::glob;
use log::{error, trace};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Local};

const RAPL_DIR: &str = "/sys/devices/virtual/powercap/intel-rapl/";

// This glob pattern picks up all the energy files for the "core" energy
// for all of the available domains (i.e. sockets) in the server
// There is an assumption that there is at least one and no more than nine domains.
// With more than 9 domains replace the question marks with asterisks.
const RAPL_CORE_GLOB: &str = "intel-rapl:?/intel-rapl:?:0/energy_uj";

// Holds the concrete (non-globbed) RAPL paths
pub struct RAPL {
    rapl_paths: Vec<PathBuf>,
}

pub struct RAPL_Reading {
    domain: u64,
    value: u64,
}

pub struct RAPL_Readings {
    timestamp: DateTime<Local>,
    rapl_readings: Vec<RAPL_Reading>,
}

impl RAPL {
    pub fn new() -> Self {
        let rapl_glob = RAPL_DIR.to_owned() + RAPL_CORE_GLOB;
        let mut paths = Vec::<PathBuf>::new();
        for path in glob(&rapl_glob).unwrap() {
            match path {
                Ok(p) => paths.push(p),
                Err(e) => error!("Failed to load RAPL path: {}", e),
            }
        }
        trace!("RAPL paths: {:#?}", paths);

        Self { rapl_paths: paths }
    }

    // I studied an alternative, more natural, implementation
    // of this method using map/reduce but it required cloning
    // the paths every time, so I opted for this approach:

    /// read_current_energy
    ///
    /// returns the sum of core energy values from all domains
    pub fn read_current_energy(&self) -> u64 {
        let mut total_energy: u64 = 0;
        for path in &self.rapl_paths {
            let reading: u64 = fs::read_to_string(path)
                .expect("Failed to read energy file")
                .trim()
                .parse()
                .expect("Failed to parse energy reading");
            total_energy += reading;
            trace!("Read energy value: {reading}");
        }
        trace!("Total energy: {total_energy}");
        total_energy
    }

    // class method
    pub fn domain_from_path() -> u64 {

        0
    }
}
