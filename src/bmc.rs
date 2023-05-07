use log::{trace, debug, info, error};
use std::process::{Command, Output};

const IPMI_PATH: &str = "/usr/bin/ipmitool";
const BMC_READ_POWER_CMD: &str = "-c sdr type 0x09";


pub struct BMC {
    pub hostname: String,
    pub username: String,
    pub password: String,
}

impl BMC {
    pub fn new(hostname: &str, username: &str, password: &str) -> Self {
        Self {
            hostname: String::from(hostname),
            username: String::from(username),
            password: String::from(password),
        }
    }

    fn run_bmc_command(&self, bmc_command: &str) -> Output {

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
            Ok(out) => out,
            Err(e) => {
                error!("Failed to execute command: {} {}: {:?}",
                    &IPMI_PATH, &ipmi_args.join(","), e);
                panic!("Can't run commands: {e:?}")
            }
        }

    }

    pub fn read_power(&self) -> u64 {
        let bmc_output = self.run_bmc_command(BMC_READ_POWER_CMD);
        let stdout = String::from_utf8_lossy(&bmc_output.stdout);
        let stderr = String::from_utf8_lossy(&bmc_output.stderr);
        debug!("BMC read_power() output: {}", &stdout);
        if stderr.len() > 0 {
            error!("Problem running BMC read_power() command: {}", &stderr);
        }

        // stdout contains two lines of CSV information
        let (line1, line2) = stdout
            .split_once("\n")
            .expect("Didn't find a newline in BMC read_power() output");

        trace!("BMC read_power() – line1: {line1}, line2: {line2}");

        // sanity check
        assert!(line1.contains("Watts"), "BMC read_power() output line1 missing Watts");
        assert!(line2.contains("Watts"), "BMC read_power() output line2 missing Watts");
        assert!(line2.contains("AVG"), "BMC read_power() output line2 missing AVG");

        // Instant power is the second field of the first line (CSV)
        let instant_power = line1
            .split(",")
            .collect::<Vec<&str>>()[1]
            .parse::<u64>()
            .expect("Failed to parse BMC read_power() output into a u64");

        info!("BMC power reading: {instant_power}");
        instant_power
    }

    #[allow(unused_variables)]
    pub fn set_cap_power_level(&self, cap: u64) {}

    pub fn activate_power_cap(&self) {}

    pub fn deactivate_power_cap(&self) {}

    pub fn capping_is_active(&self) -> bool {
        true
    }

    pub fn current_cap_level(&self) -> u64 {
        2
    }
}
