use std::{
    // io::{self, Write, Read, Result},
    fs,
};



static CONFIG_FILENAME: &str = "config/config.toml";



fn main() {
    // let path = env::current_exe().expect("Couldn't get cwd");
    // path.push(CONFIG_FILENAME);
    // println!("Looking for config in: {}", &path.display());
    let config: capping::Config = {
        let config_text = fs::read_to_string(CONFIG_FILENAME).expect("Failed to read config file");
        toml::from_str(&config_text).expect("Failed to parse config to toml")
    };
    println!("BMC hostname: {}", config.bmc.hostname);

    capping::read_sensors(&config);
}
