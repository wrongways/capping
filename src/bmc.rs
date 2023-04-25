use chrono::{self, DateTime, Local};
use std::process::{Command, Output};
use log::{info, trace, warn, error};


const IPMI_PATH: &str = "/usr/bin/ipmitool";
// const IPMI_READ_POWER_CMD: &str = "dcmi power reading";
const IPMI_READ_POWER_CMD: &str = "-c sdr type 0x09";
// const IPMI_GET_POWER_CAP_CMD: &str = "";
// const IPMI_SET_POWER_CAP_CMD: &str = "";

#[derive(Debug)]
pub struct BMC {
    hostname: String,
    username: String,
    password: String,
    pub power_readings: Vec<PowerReading>,
}


#[derive(Debug)]
pub struct PowerReading {
    pub timestamp: DateTime<Local>,
    pub instant: u32,
    pub avg: u32,
}

impl PowerReading {
    pub fn new(instant: u32, avg: u32) -> Self {
        let timestamp: DateTime<Local> = Local::now();
        Self {
            timestamp,
            instant,
            avg,
        }
    }
}

impl BMC {
    pub fn new(host: String, user: String, passwd: String) -> Self {
        Self {
            hostname: host,
            username: user,
            password: passwd,
            power_readings: Vec::with_capacity(60),
        }
    }
    pub fn working(&self) {
        trace!("It's working user: {}", self.username);
    }

    fn run_ipmi_command(&self, bmc_command: &str) -> Output {
        let ipmi_args = format!(
            "-H {} -U {} -P {} {}",
            self.hostname, self.username, self.password, bmc_command
        );

        trace!("Sensor command: {ipmi_args}");

        let ipmi_args: Vec<&str> = ipmi_args.split_whitespace().collect();
        trace!("impi_args: {:?}", ipmi_args);
        let result = Command::new(IPMI_PATH)
            .args(ipmi_args)
            .output()
            .expect("Failed to run impi command");

        result
    }

    pub fn read_sensors(&mut self) {
        let result = self.run_ipmi_command(&IPMI_READ_POWER_CMD);

        let power_csv = String::from_utf8_lossy(&result.stdout);
        let power_error = String::from_utf8_lossy(&result.stderr);
        trace!("Output from read_sensors: {power_csv}");
        trace!("Error from read_sensors: {power_error}");
        let (line1, line2) = power_csv
            .split_once('\n')
            .expect("Failed to parse power readings");
        trace!("{line1}, {line2}");
        assert!(line1.contains("Watts"), "power line1 missing Watts");
        assert!(line2.contains("Watts"), "power line2 missing Watts");
        assert!(line2.contains("AVG"), "power line2 missing AVG");

        let line1: Vec<&str> = line1.split(',').collect();
        let line2: Vec<&str> = line2.split(',').collect();

        let instant_power = line1[1].parse::<u32>().unwrap();
        let avg_power = line2[1].parse::<u32>().unwrap();

        info!("Instant power: {instant_power:04}, Average power: {avg_power:04}");
        PowerReading::new(instant_power, avg_power);
        self.power_readings.push(PowerReading::new(instant_power, avg_power));
    }
}
