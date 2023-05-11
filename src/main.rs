use crate::driver::Driver;
use log::{debug, info};
use simple_logger::SimpleLogger;
use std::{fs, sync::mpsc, thread};

use capping::cli::CONFIGURATION;
use capping::{driver, monitor};


/// `main()` - entry point
///
/// The application parses the command-line arguments (see below) creates the stats directory,
/// launches a monitor thread that drives BMC and RAPL monitors (see `monitor.rs`)
/// before launching the primary load driver (see `driver.rs`) which runs through the
/// test permutations.
///
/// Once the load driver has completed, signal the monitor threads that they should exit
/// and save their results. The main threads waits for these child threads to complete
/// before exiting itself.

fn main() {
    SimpleLogger::new().env().init().unwrap();

    // Beware, here be fairies! There's a sort of magic around the lazy initialisation
    // of the configuration. At the first access, the configuration is loaded which in
    // turn sets off the parsing of the command-line arguments, so we appear to get a
    // value without having ever initialised it.
    debug!("Runtime config\n{:#?}", *CONFIGURATION);

    // create the stats directory
    fs::create_dir_all(&CONFIGURATION.stats_dir).expect("Failed to create stats directory");

    // create channel + sender & receiver for the monitor thread
    // mpsc = multi-producer, single consumer
    let (monitor_tx, monitor_rx) = mpsc::channel();

    info!("Launching monitor");
    // the "move" here gives ownership of the monitor_rx channel to the thread
    let monitor_thread = thread::spawn(move || monitor::monitor(&monitor_rx));
    info!("Launching driver");
    let driver = Driver::new();
    driver.run();
    info!("Driver exited");

    // Signal monitor to shutdown
    monitor_tx.send(()).unwrap();

    // Wait for monitor to exit - the child threads have to write their stats
    monitor_thread.join().unwrap();
    info!("Monitor ended");
}
