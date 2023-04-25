use capping::bmc;
use capping::firestarter;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct CLI {
    hostname: String,
    username: String,
    password: String,
}

fn main() {
    let args = CLI::parse();

    let mut bmc = bmc::BMC::new(
        args.hostname,
        args.username,
        args.password,
    );
    bmc.working();
    bmc.read_sensors();
    for sensor in &bmc.power_readings {
        println!("{:?}", sensor);
    }
    firestarter::firestarter();
}
