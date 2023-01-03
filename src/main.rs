use clap::Parser;
use std::env;
use std::process::Command;
use url::Url;

#[derive(Parser, Debug)]
struct Opts {
    #[arg(long)]
    url: String,
    #[arg(long)]
    auth: Option<String>,
    #[arg(long)]
    folder: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();

    let url_str = &opts.url;
    let url = match Url::parse(url_str) {
        Ok(url) => url,
        Err(err) => {
            eprintln!("Error parsing URL: {}", err);
            std::process::exit(1);
        }
    };

    let auth = get_arg_or_env(opts.auth, "AUTH");
    let folder = get_arg_or_env(opts.folder, "FOLDER");

    let docker_command = format!(
        "docker run --rm -it -e URL={} -e AUTH={} -e FOLDER={} my-container",
        url_str, auth, folder
    );
    let output = Command::new("sh")
        .arg("-c")
        .arg(docker_command)
        .output()
        .expect("Failed to execute Docker command");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn get_arg_or_env(arg: Option<String>, env_name: &str) -> String {
    arg.unwrap_or_else(|| {
        env::var(env_name).unwrap_or_else(|_| {
            eprintln!(
                "--{} argument or {} environment variable required",
                env_name.to_lowercase(),
                env_name
            );
            std::process::exit(1);
        })
    })
}
