#[macro_use]
extern crate clap;

use crate::less::run;
use std::path::PathBuf;

mod less;

#[derive(Clap)]
#[clap(version = "0.0.1", author = "Federico Ponzi")]
struct Opts {
    #[clap(takes_value = true)]
    /// name of the file to read
    filename: Option<PathBuf>,
}

fn main() {
    let opts: Opts = Opts::parse();
    run(opts.filename).unwrap();
}
