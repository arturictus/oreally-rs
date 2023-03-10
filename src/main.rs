use clap::Parser;
use oreally::{run, Cli};

fn main() {
    let cli: Cli = Cli::parse();
    run(cli).map_err(|e| println!("{:?}", e)).unwrap();
}
