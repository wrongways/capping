use chrono::{DateTime, Local, SecondsFormat};
use glob::glob;
use log::trace;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

// There is one sub-directory of this directory for each RAPL domain - usually a
// domain maps to a socket. For each domain energy readings for the core and memory
// are available in sub-domains. The assumption, based on anecdotal evidence only,
// is that sub-domain 0 is the processor core. A more rigorous approach would read
// the name of each sub-domain to identify each part. For future work perhaps?
const RAPL_GLOB: &str = "/sys/devices/virtual/powercap/intel-rapl/intel-rapl:*";
const MAX_ENERGY_PATH: &str ="/sys/devices/virtual/powercap/intel-rapl/intel-rapl:0/max_energy_range_uj";


// Holds the concrete (non-globbed) RAPL paths
#[derive(Debug)]
pub struct RAPL {
    /// A list of fully-qualified paths for the core energy files for every domain.
    pkg_paths: HashMap<u64, PathBuf>,
    core_paths: HashMap<u64, PathBuf>,
}

impl Default for RAPL {
    fn default() -> Self {
        Self::new()
    }
}

/// Holds the energy reading for a package or core for a given socket (domain)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct RAPL_Reading {
    /// domain id: "core0", "core1", "pkg1", "pkg2",...
    pub domain: String,
    /// The reading. For the RAPL object, this reading is in µJ.
    /// The structure is also used in `monitor/monitor_rapl.rs` when converting energy to
    /// power, with units in Watts.
    pub reading: u64,
}


/// A timestamped collection of energy readings for package and cores on all sockets
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
        let mut pkg_paths = HashMap::new();
        let mut core_paths = HashMap::new();

        // This glob pattern picks up all the energy files for the "core" energy
        // for all of the available domains (i.e. sockets) in the server
        for glob_result in glob(RAPL_GLOB).expect("RAPL failed to glob directory") {
            let path = glob_result.expect("RAPL failed to read rapl directory");
            let domain = RAPL::domain_from_path(&path);
            pkg_paths.insert(domain, path.join("energy_uj"));

            let core_dir = path.file_name()
                .expect("Failed to get rapl dir")
                .to_string_lossy()
                .to_string() + ":0";
            core_paths.insert(domain, path.join(core_dir).join("energy_uj"));
        }

        trace!("RAPL pkg_paths: {pkg_paths:?}");
        trace!("RAPL core_paths: {core_paths:?}");
        Self { pkg_paths, core_paths }
    }

    /// `read_current_energy`
    ///
    /// returns list of core energy values for all domains
    #[must_use]
    pub fn read_current_energy(&self) -> RAPL_Readings {
        let mut readings: Vec<RAPL_Reading> = Vec::new();
        for (pkg_core, label) in [(&self.core_paths, "core"), (&self.pkg_paths, "pkg")] {
            for (domain_id, path) in pkg_core.iter() {
                let energy = RAPL::read_energy(path);
                let domain = format!("{label}{domain_id}");
                readings.push(RAPL_Reading::new(&domain, energy));
            }
        }
        RAPL_Readings::new(readings)
    }

    // class method
    /// Parse a RAPL path and extract the domain id.
    #[must_use]
    fn domain_from_path(path: &Path) -> u64 {
        path
            .file_name()
            .expect("RAPL failed to get directory name")
            .to_string_lossy()
            .to_string()
            .split(':')
            .nth(1)
            .expect("Didn't find a colon separator in path")
            .parse::<u64>()
            .expect("Didn't find a number after the first colon")
    }

    // 64-bits are enough - max energy typically in 36-bits
    fn read_energy(path: &PathBuf) -> u64 {
        fs::read_to_string(path)
        .expect("Failed to read energy file")
        .trim()
        .parse()
        .expect("Failed to parse energy reading")
    }

    pub fn max_energy() -> u64 {
        // The energy counters wrap-around on reaching max energy
        // this happens sufficiently frequently (typically 5-10 minutes)
        // that it has to be handled. The wrap-around value is provided
        // in the max-energy file. This routine reads and returns that value.

        fs::read_to_string(MAX_ENERGY_PATH)
            .expect("Failed to read max energy file")
            .trim()
            .parse()
            .expect("Failed to parse max energy reading")
    }

    pub fn domain_count() -> u64 {
        glob(RAPL_GLOB).expect("RAPL failed to glob directory").count() as u64
    }
}

impl RAPL_Reading {
    #[must_use]
    pub fn new(domain: &str, reading: u64) -> Self {
        Self { domain: String::from(domain), reading }
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
            .map(|reading| reading.to_string() + ",")
            .collect();

        // remove the extra comma
        readings.pop();

        write!(
            f,
            "{},{}",
            self.timestamp.to_rfc3339_opts(SecondsFormat::Millis, false),
            readings
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_domain0_from_path() {
        let rapl_filename = "/sys/devices/virtual/powercap/intel-rapl/intel-rapl:0";
        let mut rapl_path = PathBuf::new();
        rapl_path.push(rapl_filename);
        assert_eq!(RAPL::domain_from_path(&rapl_path), 0);
    }

    #[test]
    fn test_domain16_from_path() {
        let rapl_filename = "/sys/devices/virtual/powercap/intel-rapl/intel-rapl:16";
        let mut rapl_path = PathBuf::new();
        rapl_path.push(rapl_filename);
        assert_eq!(RAPL::domain_from_path(&rapl_path), 16);
    }
}
