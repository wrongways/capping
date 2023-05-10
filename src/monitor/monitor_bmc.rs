use crate::bmc::BMC;
use crate::cli::CONFIGURATION;
use crate::{save_power_stats, PowerStat};
use log::{info, trace};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

pub fn monitor_bmc(rx: &Receiver<()>) {
    info!("\tBMC: launched");
    let runtime_estimate = (CONFIGURATION.warmup_secs + CONFIGURATION.test_time_secs) * 500;
    let mut stats = Vec::<PowerStat>::with_capacity(runtime_estimate as usize);
    let bmc = BMC::new(
        &CONFIGURATION.bmc_hostname,
        &CONFIGURATION.bmc_username,
        &CONFIGURATION.bmc_password,
    );
    loop {
        if rx.try_recv().is_ok() {
            trace!("\tBMC: got message - exiting");
            break;
        }
        let power_reading = bmc.current_power();
        stats.push(PowerStat::new(power_reading));
        trace!("\tBMC READING: {}", power_reading);

        // TODO:
        //      Add check to determine if capping is applied & get current level
        //      Log these data

        // TODO:
        //      Remove this sleep
        trace!("\tBMC: sleeping");
        thread::sleep(Duration::from_millis(250));
    }

    save_power_stats("bmc", stats, "bmc_power").expect("Failed to save BMC stats");
    info!("\tBMC: Exiting");
}
