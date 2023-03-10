use clap::Parser;
use oreally::{run, Cli};

fn main() {
    let cli: Cli = Cli::parse();
    match run(cli) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
