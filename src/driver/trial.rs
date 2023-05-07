use crate::bmc::BMC;
use crate::cli::CONFIGURATION;
use crate::core_count;
use crate::driver::firestarter::Firestarter;
use crate::driver::{CappingOperation, CappingOrder};
use chrono::{DateTime, Local, SecondsFormat};
use log::trace;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::thread::{self, JoinHandle};
use std::time::Duration;

const LOG_FILENAME: &str = "driver_log";

// Let firestarter run an extra 2 seconds
// to ensure that we get a response from the
// bmc

pub struct Trial {
    bmc: BMC,
    // rapl: RAPL,
    cap_from: u64,
    cap_to: u64,
    capping_order: CappingOrder,
    capping_operation: CappingOperation,
    total_runtime_secs: u64,
    warmup_secs: u64,
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
            // rapl: RAPL::new(),
            total_runtime_secs: CONFIGURATION.test_time_secs + CONFIGURATION.warmup_secs,
            warmup_secs: CONFIGURATION.warmup_secs,
        }
    }

    pub fn run(&mut self) {
        self.run_decreasing_load();
        self.run_decreasing_threads();
    }

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

    fn run_decreasing_load(&mut self) {
        trace!("Running decreasing load");
        self.set_initial_conditions();
        // because Rust doesn't have decreasing ranges, have to jump through hoops...
        let n_threads = 0; // firestarter will use all available threads
        for idle_pct in 1..=5 {
            let load_pct = 100 - idle_pct;
            for load_period_us in [100, 1000, 10_000] {
                self.run_test_scenario(load_pct, load_period_us, n_threads);
            }
        }
    }

    fn run_decreasing_threads(&mut self) {
        trace!("Running decreasing threads");
        self.set_initial_conditions();
        let load_pct = 100;
        let load_period = 0;
        let core_count = core_count();
        // As rust _still_ doesn't have decreasing ranges, jump through more hoops
        for idle_threads in 0..=core_count / 4 {
            self.run_test_scenario(load_pct, load_period, core_count - idle_threads);
        }
    }

    fn run_test_scenario(&mut self, load_pct: u64, load_period_us: u64, n_threads: u64) {
        trace!("Test scenario: load: {load_pct}, load period µs: {load_period_us}, n_threads: {n_threads}");
        let start_time = Local::now();
        let firestarter =
            Firestarter::new(self.total_runtime_secs, load_pct, load_period_us, n_threads);
        let fire_starter_thread = thread::spawn(move || firestarter.run());
        thread::sleep(Duration::from_secs(self.warmup_secs));
        // let _initial_load_power = self.rapl.current_power_watts();

        let cap_request_time = Local::now();
        let cap_thread = self.do_cap_operation();

        // check to see if capping thread has returned
        let sleep_millis = 250;
        let mut cap_did_complete = false;
        let mut cap_complete_time_millis: i64 = -1;
        for t in 0..(CONFIGURATION.test_time_secs * 1000 / sleep_millis) {
            thread::sleep(Duration::from_millis(1000));
            if cap_thread.is_finished() {
                cap_did_complete = true;
                cap_complete_time_millis = (t * sleep_millis) as i64;
                break;
            }
        }

        // TODO: check if capping worked by comparing power just before
        // firestarter exits to the initial_load_power above. If capping worked
        // can do an early exit

        // wait for firestarter to exit
        fire_starter_thread.join().unwrap();
        let end_time = Local::now();


        self.log_results(
            start_time,
            end_time,
            cap_request_time,
            cap_complete_time_millis,
            load_pct,
            load_period_us,
            n_threads,
            cap_did_complete,
        );
    }

    fn do_cap_operation(&self) -> JoinHandle<()> {
        // get local copies of properties for the thread closure to avoid lifetime hassles
        let cap_to = self.cap_to;
        let capping_order = self.capping_order;
        let capping_operation = self.capping_operation;
        let bmc = BMC::new(&self.bmc.hostname, &self.bmc.username, &self.bmc.password);

        // There is NO semicolon at the end of this lot, because
        // this is the return value...
        thread::spawn(move || {
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
        })
    }

    fn log_results(
        &self,
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
        cap_request_time: DateTime<Local>,
        cap_complete_time_millis: i64,
        load_pct: u64,
        load_period_us: u64,
        n_threads: u64,
        cap_did_complete: bool,
    ) {
        // Build the filename - append a timestamp and ".csv"
        let timestamp: DateTime<Local> = Local::now();

        // Have to format! because timestamp.format() produces a DelayedString, incompatible with Path
        let save_filename = format!("{LOG_FILENAME}_{}.csv", timestamp.format("%y%m%d_%H%M"));
        let save_path = Path::new(&*CONFIGURATION.stats_dir).join(save_filename);

        // If the file doesn't exist, create it and write a CSV header
        if !save_path.exists() {
            let mut log_file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&save_path)
                .expect("Failed to create driver log file");
            writeln!(
                log_file,
                "start_time,end_time,cap_request_time,cap_complete_time_millis,load_pct,load_period,n_threads,cap_did_complete"
            )
            .expect("Failed to writer driver log file header");
        }

        let mut log_file = OpenOptions::new()
            .append(true)
            .open(&save_path)
            .expect("Failed to open driver log file");
        writeln!(
            log_file,
            "{},{},{},{},{load_pct},{load_period_us},{n_threads},{cap_did_complete}",
            &start_time.to_rfc3339_opts(SecondsFormat::Millis, true),
            &end_time.to_rfc3339_opts(SecondsFormat::Millis, true),
            &cap_request_time.to_rfc3339_opts(SecondsFormat::Millis, true),
            cap_complete_time_millis,
        )
        .expect("Failed to write driver log entry");
    }
}
