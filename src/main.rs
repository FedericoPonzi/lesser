#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;

use crate::less::run;
use std::path::PathBuf;

mod less;
mod reader;

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

    run(opts.filename)
        .map_err(|error| {
            error!("Error: {}", error);
            error
        })
        .unwrap();
}
