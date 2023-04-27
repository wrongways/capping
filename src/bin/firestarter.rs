use log::{info, trace};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::fmt::{self, Display, Formatter};
use simple_logger;


#[derive(Debug, Clone)]
struct Firestarter {
    path: String,
    pub runtime_secs: u64,
    pub load_pct: u32,
    pub load_period_us: u64,
    pub n_threads: u32,
}

impl Firestarter {
    pub fn new(path: &str, runtime: Duration, load_pct: u32, load_period_us: u64, n_threads: u32) -> Self {
        assert!(load_pct > 0 && load_pct <= 100);
        let mut load_period_us = load_period_us;
        let runtime_secs = runtime.as_secs();
        if (load_pct == 100) && (load_period_us == 0) {
            load_period_us = 1000 * runtime_secs;
        }

        trace!("Making a firestarter");
        // If n_threads == 0, use 1 thread per core given by the "CPU(s):" field from lscpu.
        let mut n_threads = n_threads;
        if n_threads == 0 {
            let lscpu = "/usr/bin/lscpu";
            let awk = "/usr/bin/awk";
            let awk_fs = "-F:";
            let awk_cmd = r#"/^CPU\(s\):/ {print $2}"#;

            let lscpu_child = Command::new(lscpu)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let awk_child = Command::new(awk)
                .arg(awk_fs)
                .arg(awk_cmd)
                .stdin(Stdio::from(lscpu_child.stdout.unwrap()))
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let output = awk_child.wait_with_output().unwrap();
            let cpu_count = String::from_utf8_lossy(&output.stdout)
                .parse::<u32>()
                .expect("Failed to parse cpu_count");

            trace!("Number of CPUs: {:?}", &cpu_count);
            n_threads = cpu_count;

        }
        trace!("{} --timeout {} --load {} --period {} --threads {}",
        path, runtime_secs, load_pct, load_period_us, n_threads);

        Self {path: String::from(path), runtime_secs, load_pct, load_period_us, n_threads}
    }
}

impl Display for Firestarter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} --timeout {} --load {} --period {} --threads {}",
        self.path, self.runtime_secs, self.load_pct, self.load_period_us, self.n_threads)
    }
}
pub fn firestarter() {
    let f = Firestarter::new("/bin/firestarter", Duration::from_secs(120), 99, 100, 0);
    info!("firestarter: {f}");
}

pub fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    println!("FIRESTARTER");
    trace!("* * *  F I R E S T A R T E R  * * *");
    firestarter();
}
