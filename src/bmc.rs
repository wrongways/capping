use chrono::{self, DateTime, Local};
use std::process::{Command, Output};


const IPMI_PATH: &str = "/usr/sbin/ipmitool";
const IPMI_READ_POWER_CMD: &str = "";
const IPMI_GET_POWER_CAP_CMD: &str = "";
const IPMI_SET_POWER_CAP_CMD: &str = "";

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
        println!("It's working user: {}", self.username);
    }

    fn run_ipmi_command(&self, bmc_command: &str) -> Output {
        let ipmi_args = format!(
            "-H {} -U {} -P {} {}",
            self.hostname, self.username, self.password, bmc_command
        );

        // TODO: Add debug log here

        let ipmi_args: Vec<&str> = ipmi_args.split_whitespace().collect();
        let result = Command::new("command")
            .args(ipmi_args)
            .output()
            .expect("Failed to run impi command");

        result
    }

    pub fn read_sensors(&mut self) {
        let sensor_command = format!("{} {}", IPMI_PATH, IPMI_READ_POWER_CMD);
        println!("Sensor command: {sensor_command}");
        let result = self.run_ipmi_command(&sensor_command);

        let power_csv = String::from_utf8_lossy(&result.stdout);
        println!("Output from read_sensors: {power_csv}");
        let (line1, line2) = power_csv
            .split_once('\n')
            .expect("Failed to parse power readings");
        println!("{line1}, {line2}");
        assert!(line1.contains("Watts"), "power line1 missing Watts");
        assert!(line2.contains("Watts"), "power line2 missing Watts");
        assert!(line2.contains("AVG"), "power line2 missing AVG");

        let line1: Vec<&str> = line1.split(',').collect();
        let line2: Vec<&str> = line2.split(',').collect();

        let instant_power = line1[1].parse::<u32>().unwrap();
        let avg_power = line2[1].parse::<u32>().unwrap();

        println!("Instant power: {instant_power:04}, Average power: {avg_power:04}");
        PowerReading::new(instant_power, avg_power);
        self.power_readings.push(PowerReading::new(instant_power, avg_power));
    }
}
