use capping::firestarter::Firestarter;
use log::{info, trace};
use simple_logger;


pub fn firestarter() {
    let f = Firestarter::new("/home_nfs/wainj/local/bin/firestarter", 5, 50, 100_000, 5);
    info!("firestarter: {f}");
    trace!("Launching firestarter");
    f.launch();
    trace!("Exited firestarter");
}

pub fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    trace!("* * *  F I R E S T A R T E R  * * *");
    firestarter();
}
