use clap::Parser;
use regex::bytes::Regex;
use std::env;
use std::error;
use std::process::Command;

#[derive(Parser, Debug)]
pub struct Opts {
    #[arg(long)]
    url: String,
    #[arg(long)]
    auth: Option<String>,
    #[arg(long)]
    folder: Option<String>,
}

pub fn run(opts: Opts) -> Result<(), Box<dyn error::Error>> {
    // let opts: Opts = Opts::parse();

    let url_str = &opts.url;
    let (epub_name, book_id) =
        parse_url(url_str).unwrap_or_else(|_| panic!("invalid Url: {}", url_str));

    let auth = get_arg_or_env(opts.auth, "OREALLY_AUTH")
        .unwrap_or_else(|| panic!("--auth argument or OREALLY_AUTH environment variable required"));
    let folder = get_arg_or_env(opts.folder, "OREALLY_FOLDER").unwrap_or_else(|| "~".to_string());
    let docker_command = format!(
        "(docker run kirinnee/orly:latest login {book_id} {auth}) > \"{folder}/{epub_name}.epub\"",
    );
    let output = Command::new("sh")
        .arg("-c")
        .arg(docker_command)
        .output()
        .expect("Failed to execute Docker command");

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn get_arg_or_env(arg: Option<String>, env_name: &str) -> Option<String> {
    match arg {
        Some(arg) => Some(arg),
        None => match env::var(env_name) {
            Ok(env) => Some(env),
            Err(_) => None,
        },
    }
}
fn parse_url(url_str: &str) -> Result<(String, String), Box<dyn error::Error>> {
    let re = Regex::new(r"learning.oreilly.com/library/view/(?P<name>.+)/(?P<id>\d+).*").unwrap();
    let captures = re
        .captures(url_str.as_bytes())
        .unwrap_or_else(|| panic!("invalid Url: {}", url_str));

    let id: String = std::str::from_utf8(captures.name("id").unwrap().as_bytes())
        .unwrap()
        .to_owned();
    let name: String = std::str::from_utf8(captures.name("name").unwrap().as_bytes())
        .unwrap()
        .to_owned();
    Ok((name, id))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_url() {
        let url_str = "https://learning.oreilly.com/library/view/learn-postgresql/9781838985288";
        let (title, id) = parse_url(url_str).unwrap();
        assert_eq!(title, "learn-postgresql");
        assert_eq!(id, "9781838985288");
        let url_str =
            "https://learning.oreilly.com/library/view/learn-postgresql/9781838985288/fdfd";
        let (title, id) = parse_url(url_str).unwrap();
        assert_eq!(title, "learn-postgresql");
        assert_eq!(id, "9781838985288");
    }

    #[test]
    fn test_get_arg_or_env_panics() {
        get_arg_or_env(None, "ENV");
    }
    #[test]
    fn test_get_arg_or_env() {
        assert_eq!(
            get_arg_or_env(Some("Hello".to_string()), "ENV"),
            Some("Hello".to_string())
        );
        std::env::set_var("ENV", "env_value");
        assert_eq!(get_arg_or_env(None, "ENV"), Some("env_value".to_string()));
        assert_eq!(
            get_arg_or_env(Some("Hello".to_string()), "ENV"),
            Some("Hello".to_string())
        );
    }
}
