use crate::lesser::run;
use clap::Clap;
use std::path::PathBuf;
mod lesser;

#[derive(Clap)]
#[clap(version = "0.0.1")]
struct Opts {
    #[clap(takes_value = true)]
    /// name of the file to read
    filename: Option<PathBuf>,
}

fn main() {
    // Set up logging.
    let env = env_logger::Env::new()
        .filter("LESSER_LOG")
        .write_style("LESSER_LOG_STYLE");
    env_logger::init_from_env(env);

    let opts: Opts = Opts::parse();
    if let Err(error) = run(opts.filename) {
        eprintln!("Error: {}", error);
    };
}
