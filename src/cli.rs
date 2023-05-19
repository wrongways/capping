use ::clap::Parser;
use lazy_static::lazy_static;
use chrono::Local;

const BMC_STATS_FILENAME_PREFIX: &str = "bmc_stats";
const RAPL_STATS_FILENAME_PREFIX: &str = "rapl_stats";
const DRIVER_LOG_FILENAME_PREFIX: &str = "driver_log";
const MONITOR_POLL_FREQ_HZ: u64 = 4;

lazy_static! {
    /*
        Global configuration variable.

        Lazy-static creates singleton (one-off) types that wraps a value
        providing single initialization and thread-safety.

        For a given: static ref NAME: TYPE = EXPR;
        The lazy_static macro creates a unique type that implements
        Deref<TYPE> and stores it in a static with name NAME.

        It is the wrapped value that implements any traits (eg Debug, Clone),
        NOT the wrapper. Because of this, must deref (*NAME) when debug/trace
        printing.
    */

    pub static ref CONFIGURATION: Configuration = Configuration::new();
}

#[derive(Debug)]
pub struct Configuration {
    pub bmc_hostname: String,
    pub bmc_username: String,
    pub bmc_password: String,
    pub warmup_secs: u64,
    pub test_time_secs: u64,
    pub cap_low_watts: u64,
    pub cap_high_watts: u64,
    pub stats_dir: String,
    pub bmc_stats_filename_prefix: String,
    pub rapl_stats_filename_prefix: String,
    pub driver_log_filename_prefix: String,
    pub monitor_poll_freq_hz: u64,
    pub test_timestamp: String,
    pub firestarter: String,
    pub ipmi: String,
}

impl Configuration {
    fn new() -> Self {
        let args = CLI::parse();
        let timestamp_format = "%y%m%d_%H%M";
        let test_timestamp = Local::now().format(timestamp_format).to_string();

        Configuration {
            bmc_hostname: args.bmc_hostname,
            bmc_username: args.bmc_username,
            bmc_password: args.bmc_password,
            warmup_secs: args.warmup,
            test_time_secs: args.test_time,
            cap_low_watts: args.cap_low_watts,
            cap_high_watts: args.cap_high_watts,
            stats_dir: args.stats_dir,
            bmc_stats_filename_prefix: String::from(BMC_STATS_FILENAME_PREFIX),
            rapl_stats_filename_prefix: String::from(RAPL_STATS_FILENAME_PREFIX),
            driver_log_filename_prefix: String::from(DRIVER_LOG_FILENAME_PREFIX),
            monitor_poll_freq_hz: MONITOR_POLL_FREQ_HZ,
            test_timestamp,
            firestarter: args.firestarter,
            ipmi: args.ipmi,
        }
    }
}

/*
    >>> ATTENTION <<<

        When updating the CLI structure below, you'll probably want to
        update the Configuration structure (and its implementation) too.
*/

#[allow(clippy::upper_case_acronyms)]
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct CLI {
    // Passing default values here for the tests - to to deleted
    #[arg(long, short = 'H', name = "host")]
    bmc_hostname: String,

    #[arg(long, short = 'U', name = "user")]
    bmc_username: String,

    #[arg(long, short = 'P', name = "password")]
    bmc_password: String,

    #[arg(
        long,
        default_value_t = 5,
        name = "warmup seconds",
        help = "Number of seconds to warm up before applying cap"
    )]
    warmup: u64,

    #[arg(
        long,
        short,
        default_value_t = 5,
        name = "test time seconds",
        help = "Number of seconds to wait after applying a cap before testing if cap has been applied. "
    )]
    test_time: u64,

    #[arg(
        long = "cap_low",
        short = 'w',
        default_value_t = 440,
        name = "low watts",
        help = "Number of Watts for setting a low cap"
    )]
    cap_low_watts: u64,

    #[arg(
        long = "cap_high",
        short = 'W',
        default_value_t = 550,
        name = "high watts",
        help = "Number of Watts for setting a high cap, used before setting a low cap"
    )]
    cap_high_watts: u64,

    #[arg(
        long,
        short,
        default_value = "./stats",
        name = "stats directory",
        help = "Directory to store runtime stats in"
    )]
    stats_dir: String,

    #[arg(
        long,
        default_value = "/home_nfs/wainj/local/bin/firestarter",
        name = "firestarter path",
        help = "Path to firestarter executable (relative or absolute)"
    )]
    firestarter: String,

    #[arg(
        long,
        default_value = "/usr/bin/ipmitool",
        name = "ipmi path",
        help = "Path to ipmi executable (relative or absolute)"
    )]
    ipmi: String,
}
