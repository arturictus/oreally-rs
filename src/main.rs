use clap::Parser;
use oreally::{run, Opts};

fn main() {
    let opts: Opts = Opts::parse();
    run(opts).unwrap();
}
