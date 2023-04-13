use std:: {
    process:: {
        Command,
    },
    // fs,
};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BMC {
    pub hostname: String,
    pub user: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IPMITOOL {
    path: String,
    power_sensors: String,
    get_limit: String,
    set_limit: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub bmc: BMC,
    pub ipmitool: IPMITOOL,
}


pub fn read_sensors(config: &Config) {
    let ipmi_args = format!("-H {} -U {} -P {} {}",
        &config.bmc.hostname, &config.bmc.user,
        &config.bmc.password, &config.ipmitool.power_sensors);

    println!("{ipmi_args}");

    let ipmi_args: Vec<&str> = ipmi_args.split_whitespace().collect();
    println!("{:?}", ipmi_args);
    let power_csv = Command::new(&config.ipmitool.path)
        .args(ipmi_args)
        .output()
        .expect("ipmitool power collection failed to run");

    let power_csv = String::from_utf8_lossy(&power_csv.stdout);
    let (line1, line2) = power_csv.split_once('\n').expect("Failed to parse power readings");
    println!("{line1}, {line2}");
    assert!(line1.contains("Watts"), "power line1 missing Watts");
    assert!(line2.contains("Watts"), "power line2 missing Watts");
    assert!(line2.contains("AVG"), "power line2 missing AVG");

    let line1: Vec<&str> = line1.split(',').collect();
    let line2: Vec<&str> = line2.split(',').collect();

    let instant_power = line1[1].parse::<i32>().unwrap();
    let avg_power = line2[1].parse::<i32>().unwrap();

    println!("Instant power: {instant_power:04}, Average power: {avg_power:04}");

}

