use capping::bmc;
use capping::firestarter;
use clap::Parser;
use simple_logger;
use log::info;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct CLI {
    #[arg(long, short='H')]
    hostname: String,
    #[arg(long, short='U')]
    username: String,
    #[arg(long, short='P')]
    password: String,
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let args = CLI::parse();

    let mut bmc = bmc::BMC::new(
        args.hostname,
        args.username,
        args.password,
    );
    bmc.working();
    bmc.read_sensors();
    for sensor in &bmc.power_readings {
        info!("{:?}", sensor);
    }
    firestarter::firestarter();
}
