use crate::driver::Driver;
use log::{debug, info};
use simple_logger;
use std::{fs, sync::mpsc, thread};

use capping::cli::CONFIGURATION;
use capping::{driver, monitor};

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    debug!("Runtime config\n{:#?}", *CONFIGURATION);

    // create the stats directory
    fs::create_dir_all(&CONFIGURATION.stats_dir).expect("Failed to create stats directory");

    // create channel + sender & receiver for the monitor thread
    let (monitor_tx, monitor_rx) = mpsc::channel();

    info!("Launching monitor");
    let monitor_thread = thread::spawn(move || monitor::monitor(monitor_rx));
    info!("Launching driver");
    let driver = Driver::new();
    driver.run();
    info!("Driver exited");

    // Signal monitor to shutdown
    monitor_tx.send(()).unwrap();

    // Wait for monitor to exit - the child threads have to write their stats
    monitor_thread.join().unwrap();
    info!("Monitor ended")
}
