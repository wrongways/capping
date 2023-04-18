// use std::{
//     // io::{self, Write, Read, Result},
//     fs,
// };
//
use capping::bmc;
use capping::redfish;
//
// static CONFIG_FILENAME: &str = "config/config.toml";



fn main() {
    // let path = env::current_exe().expect("Couldn't get cwd");
    // path.push(CONFIG_FILENAME);
    // println!("Looking for config in: {}", &path.display());


//     let config: capping::Config = {
//         let config_text = fs::read_to_string(CONFIG_FILENAME).expect("Failed to read config file");
//         toml::from_str(&config_text).expect("Failed to parse config to toml")
//     };
//     println!("BMC hostname: {}", config.bmc.hostname);
//
//     let power_reading = capping::read_sensors(&config);
//     println!("Current instant power: {}", power_reading.instant);

    let bmc = bmc::BMC::new(String::from("bmc"), String::from("wrongways"), String::from("pass"));
    bmc.working();

    redfish::redfish();
}
