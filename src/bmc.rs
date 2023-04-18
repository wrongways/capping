use std::process::Command;
use std::io;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BMC {
    pub hostname: String,
    pub username: String,
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

pub struct PowerReading {
    pub instant: i32,
    pub avg: i32,
}

impl PowerReading {
    pub fn new(instant: i32, avg: i32) -> Self {
        Self{instant, avg}
    }
}


impl BMC {

   pub fn new(host: String, user: String, passwd: String) -> Self {
      Self {
         hostname: host,
         username: user,
         password: passwd,
      }
   }
   pub fn working(&self) {
      println!("It's working user: {}", self.username);
   }

   fn run_ipmi_command(&self, bmc_command: &str) -> std::process::Output {
      let ipmi_args = format!("-H {} -U {} -P {} {}",
         self.hostname, self.username, self.password, bmc_command);

         // TODO: Add debug log here

         let ipmi_args: Vec<&str> = ipmi_args.split_whitespace().collect();
         let power_csv = Command::new("command")
            .args(ipmi_args)
            .output()
            .expect("Something went wrong");

         power_csv

   }

   pub fn read_sensors(config: &Config) -> PowerReading {
       let ipmi_args = format!("-H {} -U {} -P {} {}",
           &config.bmc.hostname, &config.bmc.username,
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
       PowerReading::new(instant_power, avg_power)
   }

}
