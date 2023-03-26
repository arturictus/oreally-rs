use clap::Parser;
use oreally::{run, Cli};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli: Cli = Cli::parse();
    match run(cli).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
