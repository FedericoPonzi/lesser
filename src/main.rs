#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

use crate::lesser::run;
use std::path::PathBuf;

mod lesser;

#[derive(Clap)]
#[clap(version = "0.0.1", author = "Federico Ponzi")]
struct Opts {
    #[clap(takes_value = true)]
    /// name of the file to read
    filename: Option<PathBuf>,
}

fn main() {
    let opts: Opts = Opts::parse();
    // Set up logging.
    let env = env_logger::Env::new()
        .filter("LESSER_LOG")
        .write_style("LESSER_LOG_STYLE");
    env_logger::init_from_env(env);
    if let Err(error) = run(opts.filename) {
        eprintln!("Error: {}", error);
    };
}
