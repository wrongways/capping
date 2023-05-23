use crate::bmc::BMC;
use crate::cli::CONFIGURATION;
use crate::core_count;
use crate::driver::firestarter::Firestarter;
use crate::driver::{CappingOperation, CappingOrder};
use chrono::{self, DateTime, Local, SecondsFormat};
use log::{trace, info};
use std::cmp::max;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const INTER_TRIAL_WAIT_SECS: u64 = 3; // pause between each trial - cool-down.
const SETUP_PAUSE_SECS: u64 = 1;    // Pause between issuing bmc commands in set_initial_conditions()

/// The test driver - for any given test configuration from `../driver.rs` launch a
/// campaign of tests with diminishing load. In parallel the bmc and rapl monitors
/// log the energy/power behaviour of the system under test. For each test configuration
/// three separate mechanisms of load reduction are used:
/// * Increase the period for which the load is averaged. For example for a load running
///   at 99%, calculate 99% over 100µs (a 1µs idle time per 100µs), 1000µs (10ms idle period
///   every millisecond), and 10,000µs (100µs idle period every 10ms).
/// * Decrease the average load: 99%, 98%, 97%...
/// * Run at 100% load over a diminishing number of threads (starting with all available).
pub struct Trial {
    bmc: BMC,
    cap_from: u64,
    cap_to: u64,
    capping_order: CappingOrder,
    capping_operation: CappingOperation,
    total_runtime_secs: u64,
    warmup_secs: u64,
    load_pct: u64,
    load_period_us: u64,
    n_threads: u64,
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    cap_request_time: DateTime<Local>,
    capping_thread_did_complete: bool,
    time_to_cap: chrono::Duration,
}

impl Trial {
    pub fn new(
        cap_from: u64,
        cap_to: u64,
        capping_order: CappingOrder,
        capping_operation: CappingOperation,
    ) -> Self {
        Self {
            cap_from,
            cap_to,
            capping_order,
            capping_operation,
            bmc: BMC::new(
                &CONFIGURATION.bmc_hostname,
                &CONFIGURATION.bmc_username,
                &CONFIGURATION.bmc_password,
            ),
            total_runtime_secs: CONFIGURATION.test_time_secs + CONFIGURATION.warmup_secs,
            warmup_secs: CONFIGURATION.warmup_secs,
            load_pct: 0,
            load_period_us: 0,
            n_threads: 0,
            start_time: DateTime::from(DateTime::<Local>::MIN_UTC),
            end_time: DateTime::from(DateTime::<Local>::MIN_UTC),
            cap_request_time: DateTime::from(DateTime::<Local>::MIN_UTC),
            capping_thread_did_complete: false,
            time_to_cap: chrono::Duration::max_value(),
        }
    }

    pub fn run(&mut self) {
        self.run_decreasing_load();
        self.run_decreasing_threads();
    }

    /// Setup the target system's capping configuration, ready for testing
    /// # Arguments
    /// * - sleep_secs: give some time for the capping conditions to be applied
    ///     prior to launching the test
    // There is an assumption here that the server is under low load and these
    // prepatory operations will succeed. Should be checked?

    fn set_initial_conditions(&self) {
        match self.capping_order {
            CappingOrder::LevelBeforeActivate => {
                // Set the level to the "cap_to" value, and the
                // capping activation to the opposite of the test
                //
                // Reapeat the commands as they don't always seem to be taken into account

                self.bmc.set_cap_power_level(self.cap_to);
                thread::sleep(Duration::from_secs(SETUP_PAUSE_SECS));

                match self.capping_operation {
                    CappingOperation::Activate => self.bmc.deactivate_power_cap(),
                    CappingOperation::Deactivate => self.bmc.activate_power_cap(),
                };
            }
            CappingOrder::LevelAfterActivate => {
                // set the capping level to the "cap_from" value
                // and the capping activation to the value for the test
                self.bmc.set_cap_power_level(self.cap_from);
                thread::sleep(Duration::from_secs(SETUP_PAUSE_SECS));

                match self.capping_operation {
                    CappingOperation::Activate => self.bmc.activate_power_cap(),
                    CappingOperation::Deactivate => self.bmc.deactivate_power_cap(),
                }
            }
            CappingOrder::LevelToLevel => {
                // set cap level and activate capping
                self.bmc.set_cap_power_level(self.cap_from);
                thread::sleep(Duration::from_secs(SETUP_PAUSE_SECS));
                self.bmc.activate_power_cap();
            }
        };
        thread::sleep(Duration::from_secs(SETUP_PAUSE_SECS));
    }

    /// Combines two of the diminishing load techniques: decrease average load and
    /// increase the time period over which the average is calculated (indirectly increasing
    /// the wall-clock idle period).
    fn run_decreasing_load(&mut self) {
        trace!("Running decreasing load");

        // because Rust doesn't have decreasing ranges, have to jump through hoops...
        let n_threads = 0; // firestarter will use all available threads
        for idle_pct in 0..=25 {
            let load_pct = 100 - idle_pct;
            for load_period_us in [10_000, 100_000, 1_000_000] {
                self.set_initial_conditions();
                self.run_test_scenario(load_pct, load_period_us, n_threads);
                thread::sleep(Duration::from_secs(INTER_TRIAL_WAIT_SECS));
            }
        }
    }

    /// Decrease the number of active threads. Each active thread runs at full load.
    fn run_decreasing_threads(&mut self) {
        trace!("Running decreasing threads");

        let load_pct = 100;
        let load_period = 0;
        let core_count = core_count();
        assert!(core_count > 0);
        // As rust _still_ doesn't have decreasing ranges, jump through more hoops
        if core_count > 1 {
            // TODO: Could probably have a better algorithm here - if you've got 128
            //       cores, then maybe don't have to run this 17 times to get a result.
            //       but then again, maybe you do.

            let max_idle_threads = max(1, core_count / 4);
            for idle_threads in 0..=max_idle_threads {
                self.set_initial_conditions();
                self.run_test_scenario(load_pct, load_period, core_count - idle_threads);
                thread::sleep(Duration::from_secs(INTER_TRIAL_WAIT_SECS));
            }
        } else {
            info!("Can't run decreasing cores with only one core");
        }
    }

    /// For each configuration of firestarter as established by `run_decreasing_load()`
    /// and `run_decreasing_threads()` launch firestarter on its own thread, wait for
    /// the warm uptime then apply the capping action on the BMC. Check to see if the capping action
    /// completed inside the test time. For each test run, save results to log file.
    fn run_test_scenario(&mut self, load_pct: u64, load_period_us: u64, n_threads: u64) {
        self.load_pct = load_pct;
        self.load_period_us = load_period_us;
        self.n_threads = n_threads;
        info!("\
            Test scenario: load: {load_pct}, \
            load period µs: {load_period_us}, \
            n_threads: {n_threads}, \
            cap_from: {}, \
            cap_to: {}, \
            capping_order: {}, \
            capping_operation: {}",
            self.cap_from,
            self.cap_to,
            self.capping_order,
            self.capping_operation
        );
        self.start_time = Local::now();
        let firestarter =
            Firestarter::new(self.total_runtime_secs, load_pct, load_period_us, n_threads);
        let fire_starter_thread = thread::spawn(move || firestarter.run());
        thread::sleep(Duration::from_secs(self.warmup_secs));

        self.cap_request_time = Local::now();
        self.do_cap_operation();
        self.time_to_cap =  Local::now() - self.cap_request_time;

        // wait for firestarter to exit
        fire_starter_thread.join().unwrap();
        self.end_time = Local::now();

        self.log_results().expect("Failed to write driver log entry");
    }

    /// Perform the capping action
    fn do_cap_operation(&self)  {
        let capping_order = self.capping_order;
        let capping_operation = self.capping_operation;

        match capping_order {
            CappingOrder::LevelBeforeActivate => {
                // The capping level is set by set_initial_conditions
                // just need to perform the operation
                match capping_operation {
                    CappingOperation::Activate => self.bmc.activate_power_cap(),
                    CappingOperation::Deactivate => self.bmc.deactivate_power_cap(),
                }
            }
            CappingOrder::LevelAfterActivate | CappingOrder::LevelToLevel => {
                self.bmc.set_cap_power_level(self.cap_to);
            }
        }
    }

    fn log_results(&self) -> Result<(), std::io::Error>  {
        // Build log-file path, create the file and write a csv header
        let log_file_path = Trial::make_csv_logfile_path();
        trace!("Writing results to: {log_file_path:?}");
        if log_file_path.exists() {
            trace!("Log file exists...");
        } else {
            trace!("Log file doesn't exist, creating...");
            Trial::create_csv_file(&log_file_path).expect("Failed to create driver log file");
        }



        // Save the trail run's results
        let mut log_file = OpenOptions::new()
            .append(true)
            .open(&log_file_path)
            .expect("Failed to open driver log file");
        writeln!(
            log_file,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            self.start_time.to_rfc3339_opts(SecondsFormat::Secs, false),
            self.end_time.to_rfc3339_opts(SecondsFormat::Secs, false),
            self.cap_request_time.to_rfc3339_opts(SecondsFormat::Secs, false),
            self.capping_thread_did_complete,
            self.time_to_cap.num_milliseconds(),
            self.load_pct,
            self.load_period_us,
            self.n_threads,
            self.capping_order,
            self.capping_operation,
            self.cap_from,
            self.cap_to
        )?;
        Ok(())
    }

    // Struct method
    fn create_csv_file(path: &PathBuf) -> Result<(), std::io::Error> {
        let mut log_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;

        // write csv header
        writeln!(
            log_file,
            "start_time,\
            end_time,\
            cap_request_time,\
            cap_did_complete,\
            cap_complete_time_millis,\
            load_pct,\
            load_period,\
            n_threads,\
            capping_order,\
            capping_operation,\
            cap_from,\
            cap_to"
        )?;

        Ok(())
        // File is closed when it goes out of scope
    }

    /// Build the filename - append a timestamp and ".csv"
    fn make_csv_logfile_path() -> PathBuf {
        let save_filename = format!("{}_{}.csv",
            CONFIGURATION.driver_log_filename_prefix,
            CONFIGURATION.test_timestamp);

        Path::new(&CONFIGURATION.stats_dir).join(save_filename)
    }
}
