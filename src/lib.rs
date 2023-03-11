use clap::{Parser, Subcommand};
use std::error;
mod download;
mod init;
mod storage;
use regex::bytes::Regex;
use std::{env, thread, time};
use storage::{BookRecord, Storage};

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "oreilly")]
#[command(about = "An O'really book downloader", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    #[command(about = "Download a book")]
    Download {
        #[arg(long)]
        url: String,
        #[arg(long)]
        auth: Option<String>,
        #[arg(long)]
        folder: Option<String>,
    },
    #[command(about = "Queue a book")]
    Queue {
        #[arg(long)]
        url: String,
    },
    #[command(about = "Start processing books queue")]
    Start {
        #[arg(long)]
        auth: Option<String>,
        #[arg(long)]
        folder: Option<String>,
    },
    #[command(about = "Initalises configuration")]
    Init {
        #[arg(long, short)]
        config_folder: Option<String>,
        #[arg(long, short)]
        books_folder: Option<String>,
        #[arg(long, short)]
        username: Option<String>,
        #[arg(long, short)]
        password: Option<String>,
    },
    List,
}

#[derive(Debug)]
pub struct BookRequest {
    book_id: String,
    title: String,
    auth: String,
    folder: String,
}

pub fn run(cli: Cli) -> Result<(), Box<dyn error::Error>> {
    cli.command.run()
}

impl Commands {
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Download { .. } => {
                let req = BookRequest::new(self.clone())?;
                download::run(req)
            }
            Self::Queue { url, .. } => {
                if !Self::is_ready() {
                    return Ok(());
                }
                // Validate url
                parse_url(url)?;
                let mut record = BookRecord::new(url.to_string());
                record.insert()?;
                Ok(())
            }
            Self::List => {
                if !Self::is_ready() {
                    return Ok(());
                }
                let list = BookRecord::all()?;
                println!("{:?}", list);
                Ok(())
            }
            Self::Init { .. } => {
                if Self::is_ready() {
                    println!("Configuration already exists");
                    return Ok(());
                }
                Storage::setup()?;
                Ok(())
            }
            Self::Start { .. } => loop {
                if !Self::is_ready() {
                    break Ok(());
                }
                let list = BookRecord::all()?;
                println!("Pending books {:?}", list);
                for record in list {
                    let req = BookRequest::new_from_record(&record, self)?;
                    download::run(req)?;
                    record.delete()?;
                }
                wait();
            },
        }
    }
    pub fn is_ready() -> bool {
        let ready = Storage::is_ready();
        if !ready {
            println!("Please run `oreilly init` to setup configuration");
        }
        ready
    }
    pub fn auth(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::Download { auth, .. } | Self::Start { auth, .. } => {
                let auth = get_arg_or_env(auth.clone(), "OREALLY_AUTH")
                    .ok_or("--auth argument or OREALLY_AUTH environment variable required")?;
                Ok(auth)
            }
            _ => panic!("invalid command"),
        }
    }
    pub fn folder(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::Download { folder, .. } | Self::Start { folder, .. } => {
                let folder = get_arg_or_env(folder.clone(), "OREALLY_FOLDER")
                    .unwrap_or_else(|| "~".to_string());
                Ok(folder)
            }
            _ => panic!("invalid command"),
        }
    }
}

impl BookRequest {
    fn new_from_record(
        record: &storage::BookRecord,
        cmd: &Commands,
    ) -> Result<Self, Box<dyn error::Error>> {
        let (epub_name, book_id) = parse_url(&record.url).unwrap();

        Ok(Self {
            book_id,
            title: epub_name,
            auth: cmd.auth()?,
            folder: cmd.folder()?,
        })
    }
    fn new(cmd: Commands) -> Result<BookRequest, Box<dyn error::Error>> {
        match cmd {
            Commands::Download { url, auth, folder } => Self::build(url, auth, folder),
            _ => panic!("invalid command"),
        }
    }
    fn build(
        url: String,
        auth: Option<String>,
        folder: Option<String>,
    ) -> Result<BookRequest, Box<dyn error::Error>> {
        let (epub_name, book_id) =
            parse_url(&url).unwrap_or_else(|_| panic!("invalid Url: {}", url));

        let auth = get_arg_or_env(auth, "OREALLY_AUTH")
            .ok_or("--auth argument or OREALLY_AUTH environment variable required")?;
        let folder = get_arg_or_env(folder, "OREALLY_FOLDER").unwrap_or_else(|| "~".to_string());
        Ok(BookRequest {
            book_id,
            title: epub_name,
            auth,
            folder,
        })
    }
}

fn wait() {
    let ten_millis = time::Duration::from_millis(1000);
    thread::sleep(ten_millis);
}
fn get_arg_or_env<T: Into<String>>(arg: Option<T>, env_name: &str) -> Option<String> {
    match arg {
        Some(arg) => Some(arg.into()),
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
        .ok_or(format!("invalid Url: {}", url_str))?;

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
        get_arg_or_env(None::<String>, "ENV");
    }
    #[test]
    fn test_get_arg_or_env() {
        assert_eq!(
            get_arg_or_env(Some("Hello".to_string()), "ENV"),
            Some("Hello".to_string())
        );
        std::env::set_var("ENV", "env_value");
        assert_eq!(
            get_arg_or_env(None::<String>, "ENV"),
            Some("env_value".to_string())
        );
        assert_eq!(
            get_arg_or_env(Some("Hello".to_string()), "ENV"),
            Some("Hello".to_string())
        );
    }
}
