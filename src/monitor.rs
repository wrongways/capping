mod monitor_bmc;
mod monitor_rapl;

use log::trace;
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub fn monitor(rx: &Receiver<()>) {
    trace!("MONITOR: starting");

    let (rapl_tx, rapl_rx) = mpsc::channel();
    let (bmc_tx, bmc_rx) = mpsc::channel();

    let rapl_thread = thread::spawn(move || monitor_rapl::monitor_rapl(&rapl_rx));
    let bmc_thread = thread::spawn(move || monitor_bmc::monitor_bmc(&bmc_rx));

    trace!("MONITOR: threads launched waiting for exit message from main");
    rx.recv()
        .expect("Monitor driver failed to receive message from main thread");

    trace!("MONITOR: received message - signaling children to exit");
    for (channel, thread) in [(rapl_tx, rapl_thread), (bmc_tx, bmc_thread)] {
        channel
            .send(())
            .expect("Monitor driver failed to send messaged to child monitor");
        thread
            .join()
            .expect("Monitor driver failed to join children");
    }
    trace!("MONITOR: children halted, exiting");
}
