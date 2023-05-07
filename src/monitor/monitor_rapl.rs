use crate::cli::CONFIGURATION;
use crate::rapl::RAPL;
use crate::{save_power_stats, PowerStat};
use log::{info, trace};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

pub fn monitor_rapl(rx: Receiver<()>) {
    info!("\tRAPL: launched");
    let runtime_estimate = (CONFIGURATION.warmup_secs + CONFIGURATION.test_time_secs) * 500;
    let mut stats = Vec::<PowerStat>::with_capacity(runtime_estimate as usize);
    let rapl = RAPL::new();
    loop {
        if let Ok(_) = rx.try_recv() {
            trace!("\tRAPL: got message - exiting");
            break;
        }
        stats.push(PowerStat::new(rapl.read_current_energy()));
        trace!("\tRAPL: sleeping");
        thread::sleep(Duration::from_millis(1000));
    }
    save_power_stats("rapl", stats, "energy").expect("Failed to save RAPL stats");
    info!("\tRAPL: Exiting");
}
