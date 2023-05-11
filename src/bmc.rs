use chrono::NaiveDateTime;
use log::{debug, error, trace};
use std::process::Command;
use std::fmt::{self, Display, Debug};

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

impl Default for BMC_PowerReading {
    fn default() -> Self {
        BMC_PowerReading::new()
    }
}

#[allow(non_camel_case_types)]
pub struct BMC_CapSetting {
    pub is_active: bool,
    pub power_limit: u64,
}

impl BMC {
    #[must_use]
    pub fn new(hostname: &str, username: &str, password: &str) -> Self {
        Self {
            hostname: String::from(hostname),
            username: String::from(username),
            password: String::from(password),
        }
    }

    /// `run_bmc_command`
    ///
    /// Executes an IPMI command to run an operation on a BMC. It uses the BMC credentials configured
    /// when the self instance was created.
    ///
    /// # Arguments
    /// * `bmc_command` - a string slice with command to exectue
    ///
    /// # Return
    /// * <stdout> as a string
    ///
    /// # Panics
    /// The method will panic if the command fails to run
    fn run_bmc_command(&self, bmc_command: &str) -> String {
        // Concatenate command with the credentials
        let ipmi_args = format!("{} {}", self, bmc_command);
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

    // Capping management
    /// Returns the current cap power limit and activation state in a `CapSetting` struct
    #[must_use]
    pub fn current_cap_settings(&self) -> BMC_CapSetting {
        let bmc_output = self.run_bmc_command(BMC_CAP_SETTINGS_CMD);
        debug!("BMC cap level output\n{bmc_output}");
        BMC::parse_cap_settings(&bmc_output)
    }

    #[must_use]
    pub fn capping_is_active(&self) -> bool {
        self.current_cap_settings().is_active
    }

    #[must_use]
    pub fn current_power_limit(&self) -> u64 {
        self.current_cap_settings().power_limit
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


    // Power management
    #[must_use]
    pub fn current_power(&self) -> u64 {
        let bmc_output = self.run_bmc_command(BMC_READ_POWER_CMD);
        debug!("BMC power output:\n{bmc_output}");
        BMC::parse_power_reading(&bmc_output).instant
    }




    /// Parses a u64 from the first word in the `power_reading` string
    /// Used in the application to parse the power values returned from
    /// # Example
    /// ```
    /// use capping::bmc::BMC;
    /// assert_eq!(220, BMC::parse_number("220 Watts"));
    /// ```
    ///
    /// # Panics
    /// If the passed string is empty, or first word is not a number
    #[must_use]
    pub fn parse_number(power_reading: &str) -> u64 {
        trace!("BMC::parse_number({power_reading})");
        let parts: Vec<&str> = power_reading.trim().split_ascii_whitespace().collect();
        trace!("BMC::parse_number parts: {parts:#?}");
        assert!(!parts.is_empty());
        let n: u64 = parts[0].parse().expect("Failed to parse power reading");
        trace!("BMC::parse_number -> {n}");
        n
    }

    /// Parses a BMC date string into local time without timezone (`NaiveDateTime`)
    ///
    /// # Example
    /// ```
    /// use chrono::{NaiveDate, NaiveDateTime};
    /// use capping::bmc::BMC;
    ///
    /// let bmc_date_string = "Tue May  9 14:24:36 2023";
    /// let bmc_date = BMC::date_from_string(bmc_date_string);
    /// let expected: NaiveDateTime = NaiveDate::from_ymd_opt(2023, 5, 9).unwrap().and_hms_opt(14, 24, 36).unwrap();
    /// assert_eq!(expected, bmc_date);
    /// ```
    ///
    /// # Panics
    /// Will panic if the date string cannot be parsed
    #[must_use]
    pub fn date_from_string(date_string: &str) -> NaiveDateTime {
        // Tue May  9 14:24:36 2023
        let bmc_timestamp_fmt = "%a %b %e %H:%M:%S %Y";
        let dt = NaiveDateTime::parse_from_str(date_string.trim(), bmc_timestamp_fmt)
            .expect("Failed to parse BMC timestamp");
        trace!("BMC::date_from_string({date_string}) -> {dt}");
        dt
    }

    /// Parses the ouptut of BMC ipmi dcmi power command, returning a `BMC_PowerReading` struct
    #[must_use]
    #[allow(clippy::match_on_vec_items)]
    fn parse_power_reading(output: &str) -> BMC_PowerReading {
        let mut readings = BMC_PowerReading::new();

        // An example of the output format is shown in the tests below
        // It comprises a series of rows, some empty. The non-empty rows contain
        // a key and a value separated by a colon followed by whitespace.
        // The routine matches on the first word of the key and calls the appropriate
        // parser to convert the value into its natural type (from a string).
        for line in &mut output.lines() {
            // Can't use a simple colon (:) for the split here because of the date string
            let parts: Vec<&str> = line.trim().split(": ").collect();

            // skip empty lines by checking # parts
            if parts.len() == 2 {
                let (lhs, rhs) = (parts[0], parts[1]);
                let lhs_parts: Vec<&str> = lhs.split_ascii_whitespace().collect();
                assert!(!lhs_parts.is_empty());
                println!("BMC::parse_power_reading() parsing: {}", lhs_parts[0]);

                // despite what clippy says, this won't panic
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

    /// Parses the output of the IPMI dcmi power capping commands
    /// Like `power_readings` above, the output comprises key/value pairs separated by a colon
    // Example output is shown in the tests below.
    #[must_use]
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

impl Display for BMC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "-H {} -U {} -P {}",
            self.hostname, self.username, self.username)
    }
}

impl Debug for BMC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "-H {} -U {} -P {}",
            self.hostname, self.username, "****")
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
