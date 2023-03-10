use clap::Parser;
use oreally::{run, Cli};

fn main() {
    let opts: Cli = Cli::parse();
    run(opts).unwrap();
}
