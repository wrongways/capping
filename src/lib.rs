use std:: {
    process:: {
        Command,
    }
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
    let _power_csv = Command::new(&config.ipmitool.path)
        .args(ipmi_args)
        .output()
        .expect("ipmitool power collection failed to run");
}
