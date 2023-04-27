use capping::bmc;
use clap::Parser;
use log::info;
use simple_logger;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct CLI {
    #[arg(long, short = 'H')]
    hostname: String,
    #[arg(long, short = 'U')]
    username: String,
    #[arg(long, short = 'P')]
    password: String,
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let args = CLI::parse();

    let mut bmc = bmc::BMC::new(args.hostname, args.username, args.password);
    bmc.working();

    for _ in 0..20 {
        bmc.read_sensors();
    }

    for sensor in &bmc.power_readings {
        info!("{:?}", sensor);
    }
}
