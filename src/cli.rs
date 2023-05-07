use ::clap::Parser;
use lazy_static::lazy_static;

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
}

impl Configuration {
    fn new() -> Self {
        let args = CLI::parse();
        Configuration {
            bmc_hostname: args.bmc_hostname,
            bmc_username: args.bmc_username,
            bmc_password: args.bmc_password,
            warmup_secs: args.warmup,
            test_time_secs: args.test_time,
            cap_low_watts: args.cap_low_watts,
            cap_high_watts: args.cap_high_watts,
            stats_dir: args.stats_dir,
        }
    }
}

/*
  >>> ATTENTION <<<

    When updating this structure, you probably want to update
    the Configuration structure (and its implementation) too.
*/

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct CLI {
    // Passing default values here for the tests - to to deleted
    #[arg(long, short = 'H', name = "host", default_value = "host")]
    bmc_hostname: String,
    #[arg(long, short = 'U', name = "user", default_value = "user")]
    bmc_username: String,
    #[arg(long, short = 'P', name = "passwd", default_value = "pass")]
    bmc_password: String,
    #[arg(
        long,
        default_value_t = 3,
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
        default_value_t = 300,
        name = "low watts",
        help = "Number of Watts for setting a low cap"
    )]
    cap_low_watts: u64,
    #[arg(
        long = "cap_high",
        short = 'W',
        default_value_t = 3000,
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
}
