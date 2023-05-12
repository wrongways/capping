use crate::bmc::BMC;
use crate::cli::CONFIGURATION;
use crate::core_count;
use crate::driver::firestarter::Firestarter;
use crate::driver::{CappingOperation, CappingOrder};
use chrono::{self, DateTime, Local};
use log::{trace, info};
use std::cmp::max;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};
use std::time::Duration;


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

    // TODO: Add start/stop timestamps
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
            bmc: BMC::new("host", "user", "pass"),
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
    fn set_initial_conditions(&self) {
        match self.capping_order {
            CappingOrder::LevelBeforeActivate => {
                // Set the level to the "cap_to" value, and the
                // capping activation to the opposite of the test
                self.bmc.set_cap_power_level(self.cap_to);

                match self.capping_operation {
                    CappingOperation::Activate => self.bmc.deactivate_power_cap(),
                    CappingOperation::Deactivate => self.bmc.activate_power_cap(),
                }
            }
            CappingOrder::LevelAfterActivate => {
                // set the capping level to the "cap_from" value
                // and the capping activation to the value for the test
                self.bmc.set_cap_power_level(self.cap_from);

                match self.capping_operation {
                    CappingOperation::Activate => self.bmc.activate_power_cap(),
                    CappingOperation::Deactivate => self.bmc.deactivate_power_cap(),
                }
            }
        }
    }

    /// Combines two of the diminishing load techniques: decrease average load and
    /// increase the time period over which the average is calculated (indirectly increasing
    /// the wall-clock idle period).
    fn run_decreasing_load(&mut self) {
        trace!("Running decreasing load");
        self.set_initial_conditions();
        // because Rust doesn't have decreasing ranges, have to jump through hoops...
        let n_threads = 0; // firestarter will use all available threads
        for idle_pct in 1..=2 {
            let load_pct = 100 - idle_pct;
            for load_period_us in [100, 1000, 10_000] {
                self.run_test_scenario(load_pct, load_period_us, n_threads);
            }
        }
    }

    /// Decrease the number of active threads. Each active thread runs at full load.
    fn run_decreasing_threads(&mut self) {
        trace!("Running decreasing threads");
        self.set_initial_conditions();
        let load_pct = 100;
        let load_period = 0;
        let core_count = core_count();
        assert!(core_count > 0);
        // As rust _still_ doesn't have decreasing ranges, jump through more hoops
        if core_count > 1 {
            // TODO: Could probably have a better algorithm here - if you've got 128
            //       cores, then maybe don't have to run this 32 times to get a result.
            //       but then again, maybe you do.

            // TODO for testing purposes only
            let max_idle_threads = 2; // max(1, core_count / 4);
            for idle_threads in 0..max_idle_threads {
                self.run_test_scenario(load_pct, load_period, core_count - idle_threads);
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
        trace!("Test scenario: load: {load_pct}, load period µs: {load_period_us}, n_threads: {n_threads}");
        self.start_time = Local::now();
        let firestarter =
            Firestarter::new(self.total_runtime_secs, load_pct, load_period_us, n_threads);
        let fire_starter_thread = thread::spawn(move || firestarter.run());
        thread::sleep(Duration::from_secs(self.warmup_secs));

        self.cap_request_time = Local::now();
        let cap_thread = self.do_cap_operation();

        // check to see if capping thread has returned
        //
        // Using Futures would be a more satisfactory approach but bringing in a whole new
        // runtime, having an async main and all the additional complexity seems like a lot
        // of trouble for this single, simple case.
        //
        // An alternative technique would be to cap in the main thread, suspending until completion
        // but then need to manage the case where the capping operation overruns the end of the
        // firestarter load.
        let sleep_millis = 250;
        for _ in 0..(CONFIGURATION.test_time_secs * 1000 / sleep_millis) {
            thread::sleep(Duration::from_millis(sleep_millis));
            if cap_thread.is_finished() {
                self.capping_thread_did_complete = true;
                // capping thread returns its runtime as a Duration
                self.time_to_cap = cap_thread.join().expect("Failed to join capping thread");
                break;
            }
        }

        // TODO: check if capping worked by comparing power just before
        // firestarter exits to the initial_load_power above. If capping worked
        // can do an early exit the trial

        // wait for firestarter to exit
        fire_starter_thread.join().unwrap();
        self.end_time = Local::now();

        self.log_results().expect("Failed to write driver log entry");
    }

    /// Launch a separate thread with a request to the BMC to perform the capping action
    fn do_cap_operation(&self) -> JoinHandle<chrono::Duration> {
        // get local copies of properties to move into the thread closure
        let cap_to = self.cap_to;
        let capping_order = self.capping_order;
        let capping_operation = self.capping_operation;

        let bmc = BMC::new(&self.bmc.hostname, &self.bmc.username, &self.bmc.password);

        // There is NO semicolon at the end of this lot, because this thread join handle is the
        // return value to the do_cap_operation method. The thread itself returns its own runtime duration
        thread::spawn(move || {
            let cap_start_time = Local::now();
            match capping_order {
                CappingOrder::LevelBeforeActivate => {
                    // The capping level is set by set_initial_conditions
                    // just need to perform the operation
                    match capping_operation {
                        CappingOperation::Activate => bmc.activate_power_cap(),
                        CappingOperation::Deactivate => bmc.deactivate_power_cap(),
                    }
                }
                CappingOrder::LevelAfterActivate => {
                    // The main driver ensures that the capping operation is "Activate"
                    // as there's no sense in setting a cap when capping is deactivated
                    bmc.set_cap_power_level(cap_to);
                }
            }
            // Thread returns its execution time
            Local::now() - cap_start_time
        })
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
            "{},{},{},{},{},{},{},{},{},{}",
            self.start_time.to_rfc3339(),
            self.end_time.to_rfc3339(),
            self.cap_request_time.to_rfc3339(),
            self.capping_thread_did_complete,
            self.time_to_cap.num_milliseconds(),
            self.load_pct,
            self.load_period_us,
            self.n_threads,
            self.capping_order,
            self.capping_operation
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
            capping_operation"
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
