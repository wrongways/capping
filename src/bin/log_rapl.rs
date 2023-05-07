
const RAPL_DIR: &str = "/sys/devices/virtual/powercap/intel-rapl/";
const RAPL_CORE_GLOB: &str = "intel-rapl:?/intel-rapl:?:0/energy_uj";
const STATS_FILE: &str = "rapl_stats.csv";


fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let rapl_glob = format!("{RAPL_DIR}{RAPL_CORE_GLOB}");
    let rapl_paths = glob(&rapl_glob).expect("Failed to read rapl glob");
    let mut rapl_files = Vec::<String>::new();
    for p in rapl_paths {
        match p {
            Ok(path) => {
                let path = path.into_os_string().into_string().unwrap();
                rapl_files.push(path);
            },
            Err(why) => debug!("Titsup on path: {why:?}"),
        }
    }
    let mut energy_readings = Vec::<RAPL_Data>::new();

    for i in 0..5 {
        for rapl_file in &rapl_files {
            let rapl_domain = domain_from_path(rapl_file);
            debug!("Pushing new energy reading: {i}");
            energy_readings.push(RAPL_Data::new(rapl_domain, read_energy(rapl_file)));
        }
        sleep(Duration::from_secs(1));
    }

    trace!("Saving stats");
    write_stats(&energy_readings);
}
