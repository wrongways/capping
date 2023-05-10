use chrono::NaiveDateTime;
use log::{debug, error, trace};
use std::process::Command;

const IPMI_PATH: &str = "/usr/bin/ipmitool";
const BMC_READ_POWER_CMD: &str = "dcmi power reading";
const BMC_CAP_SETTINGS_CMD: &str = "dcmi power get_limit";
const BMC_SET_CAP_CMD: &str = "dcmi power set_limit limit 2000";
const BMC_ACTIVATE_CAP_CMD: &str = "dcmi power activate";
const BMC_DEACTIVATE_CAP_CMD: &str = "dcmi power deactivate";

pub struct BMC {
    pub hostname: String,
    pub username: String,
    pub password: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
struct BMC_PowerReading {
    instant: u64,
    minimum: u64,
    maximum: u64,
    average: u64,
    timestamp: NaiveDateTime,
}

impl BMC_PowerReading {
    pub fn new() -> Self {
        Self {
            instant: 0,
            minimum: 0,
            maximum: 0,
            average: 0,
            timestamp: NaiveDateTime::MIN,
        }
    }
}

#[allow(non_camel_case_types)]
struct BMC_CapSetting {
    is_active: bool,
    power_limit: u64,
}

impl BMC {
    pub fn new(hostname: &str, username: &str, password: &str) -> Self {
        Self {
            hostname: String::from(hostname),
            username: String::from(username),
            password: String::from(password),
        }
    }

    fn run_bmc_command(&self, bmc_command: &str) -> String {
        // Concatenate command with the credentials
        let ipmi_args = format!(
            "-H {} -U {} -P {} {}",
            self.hostname, self.username, self.password, bmc_command
        );
        debug!("BMC command: {IPMI_PATH} {ipmi_args}");

        // process::Command requires arguments as an array
        let ipmi_args: Vec<&str> = ipmi_args.split_whitespace().collect();
        trace!("bmc command arguments: {:?}", &ipmi_args);

        // Launch the command
        match Command::new(IPMI_PATH).args(&ipmi_args).output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);

                if stderr.len() > 0 {
                    error!("Problem running BMC read_power() command: {}", &stderr);
                }

                String::from(stdout)
            }
            Err(e) => {
                error!(
                    "Failed to execute command: {} {}: {:?}",
                    IPMI_PATH,
                    &ipmi_args.join(","),
                    e
                );
                panic!(
                    "Can't run command: {} {}: {e:?}",
                    IPMI_PATH,
                    &ipmi_args.join(",")
                );
            }
        }
    }

    pub fn current_power(&self) -> u64 {
        let bmc_output = self.run_bmc_command(BMC_READ_POWER_CMD);
        debug!("BMC power output:\n{bmc_output}");
        BMC::parse_power_reading(&bmc_output).instant
    }

    pub fn set_cap_power_level(&self, cap: u64) {
        let cap_cmd = format!("{BMC_SET_CAP_CMD} {cap}");
        self.run_bmc_command(&cap_cmd);
        // TODO: check the output
    }

    pub fn activate_power_cap(&self) {
        self.run_bmc_command(BMC_ACTIVATE_CAP_CMD);
    }

    pub fn deactivate_power_cap(&self) {
        self.run_bmc_command(BMC_DEACTIVATE_CAP_CMD);
    }

    pub fn capping_is_active(&self) -> bool {
        let bmc_output = self.run_bmc_command(BMC_CAP_SETTINGS_CMD);
        debug!("BMC cap level output\n{bmc_output}");
        BMC::parse_cap_settings(&bmc_output).is_active
    }

    pub fn current_power_limit(&self) -> u64 {
        let bmc_output = self.run_bmc_command(BMC_CAP_SETTINGS_CMD);
        debug!("BMC cap level output\n{bmc_output}");
        BMC::parse_cap_settings(&bmc_output).power_limit
    }

    fn parse_number(power_reading: &str) -> u64 {
        trace!("BMC::parse_number({power_reading})");
        let parts: Vec<&str> = power_reading.trim().split(' ').collect();
        trace!("BMC::parse_number parts: {parts:#?}");
        assert_eq!(parts.len(), 2);
        let rc = parts[0].parse().expect("Failed to parse power reading");
        trace!("BMC::parse_number -> {rc}");
        rc
    }

    fn date_from_string(date_string: &str) -> NaiveDateTime {
        // Tue May  9 14:24:36 2023
        let bmc_timestamp_fmt = "%a %b %e %H:%M:%S %Y";
        let rc = NaiveDateTime::parse_from_str(date_string.trim(), bmc_timestamp_fmt)
            .expect("Failed to parse BMC timestamp");
        trace!("BMC::date_from_string({date_string}) -> {rc}");
        rc
    }

    fn parse_power_reading(output: &str) -> BMC_PowerReading {
        let mut readings = BMC_PowerReading::new();

        for line in &mut output.lines() {
            // Can't use a simple colon (:) for the split here because of the date string
            let parts: Vec<&str> = line.trim().split(": ").collect();
            if parts.len() == 2 {
                let (lhs, rhs) = (parts[0], parts[1]);
                let lhs_parts: Vec<&str> = lhs.split(' ').collect();
                debug_assert!(!lhs_parts.is_empty());
                println!("BMC::parse_power_reading() parsing: {}", lhs_parts[0]);
                match lhs_parts[0] {
                    "Instantaneous" => readings.instant = BMC::parse_number(rhs.trim()),
                    "Minimum" => readings.minimum = BMC::parse_number(rhs.trim()),
                    "Maximum" => readings.maximum = BMC::parse_number(rhs.trim()),
                    "Average" => readings.average = BMC::parse_number(rhs.trim()),
                    "IPMI" => readings.timestamp = BMC::date_from_string(rhs.trim()),
                    _ => continue,
                };
            }
        }
        readings
    }

    fn parse_cap_settings(output: &str) -> BMC_CapSetting {
        // have to initialize here to keep the compiler happy
        let mut is_active: bool = false;
        let mut power_limit: u64 = 0;

        for line in &mut output.lines() {
            let parts: Vec<&str> = line.trim().split(':').collect();
            if parts.len() == 2 {
                let (lhs, rhs) = (parts[0], parts[1]);
                match lhs {
                    "Current Limit State" => is_active = rhs.trim() == "Power Limit Active",
                    "Power Limit" => power_limit = BMC::parse_number(rhs),
                    _ => continue,
                }
            }
        }
        BMC_CapSetting {
            is_active,
            power_limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_power_reading() {
        let bmc_output = "
        Instantaneous power reading:                   220 Watts
        Minimum during sampling period:                 70 Watts
        Maximum during sampling period:                600 Watts
        Average power reading over sample period:      220 Watts
        IPMI timestamp:                           Tue May  9 14:24:36 2023
        Sampling period:                          00000005 Seconds.
        Power reading state is:                   activated
        ";

        let readings = BMC::parse_power_reading(bmc_output);
        let expected_timestamp =
            NaiveDateTime::parse_from_str("2023 May 09 14:24:36", "%Y %b %d %H:%M:%S").unwrap();
        assert_eq!(readings.instant, 220);
        assert_eq!(readings.minimum, 70);
        assert_eq!(readings.maximum, 600);
        assert_eq!(readings.average, 220);
        assert_eq!(readings.timestamp, expected_timestamp);
    }

    #[test]
    fn test_parse_cap_settings_inactive() {
        let bmc_output = "
        Current Limit State: No Active Power Limit
        Exception actions:   Hard Power Off & Log Event to SEL
        Power Limit:         1600 Watts
        Correction time:     1000 milliseconds
        Sampling period:     5 seconds
        ";

        let reading = BMC::parse_cap_settings(bmc_output);
        assert!(!reading.is_active);
        assert_eq!(reading.power_limit, 1600);
    }

    #[test]
    fn test_parse_cap_settings_active() {
        let bmc_output = "
        Current Limit State: Power Limit Active
        Exception actions:   Hard Power Off & Log Event to SEL
        Power Limit:         2000 Watts
        Correction time:     1000 milliseconds
        Sampling period:     5 seconds
        ";

        let reading = BMC::parse_cap_settings(bmc_output);
        assert!(reading.is_active);
        assert_eq!(reading.power_limit, 2000);
    }
}
