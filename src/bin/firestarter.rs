use log::{info, trace};
use std::process::Command;
use std::fmt::{self, Display, Formatter};
use simple_logger;


#[derive(Debug, Clone)]
struct Firestarter {
    path: String,
    pub runtime_secs: u32,
    pub load_pct: u32,
    pub load_period_us: u64,
    pub n_threads: u32,
}

impl Firestarter {
    pub fn new(path: &str, runtime_secs: u32, load_pct: u32, load_period_us: u64, n_threads: u32) -> Self {
        assert!(load_pct > 0 && load_pct <= 100);
        Self {path: String::from(path), runtime_secs, load_pct, load_period_us, n_threads}
    }

    pub fn launch(&self) {
        let firestarter = Command::new(&self.path)
            .arg("--quiet")
            .arg("--timeout")
            .arg(self.runtime_secs.to_string())
            .arg("--load")
            .arg(self.load_pct.to_string())
            .arg("--period")
            .arg(self.load_period_us.to_string())
            .arg("--threads")
            .arg(self.n_threads.to_string())
            .spawn()
            .unwrap();

        let _ = firestarter.wait_with_output().expect("firestarter failed");
    }
}

impl Display for Firestarter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} --timeout {} --load {} --period {} --threads {}",
        self.path, self.runtime_secs, self.load_pct, self.load_period_us, self.n_threads)
    }
}
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
