use chrono::{DateTime, Local, SecondsFormat};
use glob::glob;
use log::{error, trace};
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

const RAPL_DIR: &str = "/sys/devices/virtual/powercap/intel-rapl/";

// This glob pattern picks up all the energy files for the "core" energy
// for all of the available domains (i.e. sockets) in the server
// There is an assumption that there is at least one and no more than nine domains.
// With more than 9 domains replace the question marks with asterisks.
const RAPL_CORE_GLOB: &str = "intel-rapl:?/intel-rapl:?:0/energy_uj";

// Holds the concrete (non-globbed) RAPL paths
#[derive(Debug)]
pub struct RAPL {
    rapl_paths: Vec<PathBuf>,
}

impl Default for RAPL {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub struct RAPL_Reading {
    pub domain: u64,
    pub energy: u64,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct RAPL_Readings {
    pub timestamp: DateTime<Local>,
    pub readings: Vec<RAPL_Reading>,
}

impl RAPL {
    pub fn new() -> Self {
        let rapl_glob = RAPL_DIR.to_owned() + RAPL_CORE_GLOB;
        let mut paths = Vec::<PathBuf>::new();
        for path in glob(&rapl_glob).expect("RAPL failed to get rapl virtual device") {
            match path {
                Ok(p) => paths.push(p),
                Err(e) => error!("Failed to load RAPL path: {}", e),
            }
        }
        trace!("RAPL paths: {:#?}", paths);

        Self { rapl_paths: paths }
    }

    /// `read_current_energy`
    ///
    /// returns the sum of core energy values from all domains
    pub fn read_current_energy(&self) -> RAPL_Readings {
        let mut readings: Vec<RAPL_Reading> = Vec::new();
        for path_buf in &self.rapl_paths {
            let path = path_buf.as_path();
            let energy: u64 = fs::read_to_string(path)
                .expect("Failed to read energy file")
                .trim()
                .parse()
                .expect("Failed to parse energy reading");
            let reading = RAPL_Reading::new(RAPL::domain_from_path(path), energy);
            trace!("Read energy value: {:?}", &reading);
            readings.push(reading);
        }
        let readings = RAPL_Readings::new(readings);
        trace!("Energy readings: {readings:?}");
        readings
    }

    // class method
    pub fn domain_from_path(path: &Path) -> u64 {
        path.parent()
            .expect("No parent found")
            .file_name()
            .expect("Failed to get parent directory name")
            .to_os_string()
            .into_string()
            .expect("Failed to convert path into string")
            .split(':')
            .nth(1)
            .expect("Didn't find a colon separator in path")
            .parse::<u64>()
            .expect("Didn't find a number after the first colon")
    }
}

impl RAPL_Reading {
    pub fn new(domain: u64, energy: u64) -> Self {
        Self { domain, energy }
    }
}

impl Display for RAPL_Reading {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{},{}", self.domain, self.energy)
    }
}

impl RAPL_Readings {
    pub fn new(readings: Vec<RAPL_Reading>) -> Self {
        Self {
            timestamp: Local::now(),
            readings,
        }
    }
}

impl Display for RAPL_Readings {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // this leaves the string with a trailing comma...
        let mut readings: String = self
            .readings
            .iter()
            .map(|&reading| reading.to_string() + ",")
            .collect();

        // remove the extra comma
        readings.pop();

        write!(
            f,
            "{},{}",
            self.timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
            readings
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_domain0_from_path() {
        let rapl_filename = RAPL_DIR.to_owned() + "intel-rapl:0/intel-rapl:0:0/energy_uj";
        let mut rapl_path = PathBuf::new();
        rapl_path.push(rapl_filename);
        assert_eq!(RAPL::domain_from_path(&rapl_path), 0);
    }

    #[test]
    fn test_domain1_from_path() {
        let rapl_filename = RAPL_DIR.to_owned() + "intel-rapl:1/intel-rapl:1:0/energy_uj";
        let mut rapl_path = PathBuf::new();
        rapl_path.push(rapl_filename);
        assert_eq!(RAPL::domain_from_path(&rapl_path), 1);
    }

    #[test]
    fn test_domain16_from_path() {
        let rapl_filename = RAPL_DIR.to_owned() + "intel-rapl:16/intel-rapl:16:0/energy_uj";
        let mut rapl_path = PathBuf::new();
        rapl_path.push(rapl_filename);
        assert_eq!(RAPL::domain_from_path(&rapl_path), 16);
    }
}
