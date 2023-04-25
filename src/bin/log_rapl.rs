use glob::glob;
use log::{trace, error};

const RAPL_DIR: &str = "/sys/devices/virtual/powercap/intel-rapl/";
const RAPL_CORE_GLOB: &str = "intel-rapl:?/intel-rapl:?:0/energy_uj";

fn main() {
   let rapl_glob = format!("{RAPL_DIR}{RAPL_CORE_GLOB}");
   for rapl_file in glob(&rapl_glob).expect("Failed to read rapl glob") {
      match rapl_file {
          Ok(path) => trace!("{:?}", path.display()),
          Err(e) => error!("Something went wrong: {:?}", e),
      }
   }
}
