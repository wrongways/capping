use chrono::{DateTime, Local, SecondsFormat};
use glob::glob;
use log::{error, trace};
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

// There is one sub-directory of this directory for each RAPL domain - usually a
// domain maps to a socket. For each domain energy readings for the core and memory
// are available in sub-domains. The assumption, based on anecdotal evidence only,
// is that sub-domain 0 is the processor core. A more rigorous approach would read
// the name of each sub-domain to identify each part. For future work perhaps?
const RAPL_DIR: &str = "/sys/devices/virtual/powercap/intel-rapl/";


// Holds the concrete (non-globbed) RAPL paths
#[derive(Debug)]
pub struct RAPL {
    /// A list of fully-qualified paths for the core energy files for every domain.
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
    /// domain id: 0, 1, 2,...
    pub domain: u64,
    /// The reading. For the RAPL object, this reading is in µJ.
    /// The structure is also used in `monitor/monitor_rapl.rs` when converting energy to
    /// power, with units in Watts.
    pub reading: f64,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct RAPL_Readings {
    pub timestamp: DateTime<Local>,
    /// List of readings (see above) for all known domains
    pub readings: Vec<RAPL_Reading>,
}

impl RAPL {
    #[must_use]
    pub fn new() -> Self {
        // This glob pattern picks up all the energy files for the "core" energy
        // for all of the available domains (i.e. sockets) in the server
        // The shell glob is is limited (no regex). The contents of the two starred fields
        // should be identical (this is not checked)
        const RAPL_CORE_GLOB: &str = "intel-rapl:*/intel-rapl:*:0/energy_uj";

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
    /// returns list of core energy values for all domains
    #[must_use]
    pub fn read_current_energy(&self) -> RAPL_Readings {
        let mut readings: Vec<RAPL_Reading> = Vec::new();
        for path_buf in &self.rapl_paths {
            let path = path_buf.as_path();
            let energy: f64 = fs::read_to_string(path)
                .expect("Failed to read energy file")
                .trim()
                .parse()
                .expect("Failed to parse energy reading");
            let reading = RAPL_Reading::new(RAPL::domain_from_path(path), energy);
            readings.push(reading);
        }
        RAPL_Readings::new(readings)
    }

    // class method
    /// Parse a RAPL path and extract the domain id.
    #[must_use]
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
    #[must_use]
    pub fn new(domain: u64, reading: f64) -> Self {
        Self { domain, reading }
    }
}


impl Display for RAPL_Reading {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{},{}", self.domain, self.reading)
    }
}

impl RAPL_Readings {
    #[must_use]
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
