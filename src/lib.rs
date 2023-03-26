use clap::{Parser, Subcommand};
use std::error;
mod download;
mod pueue;
mod storage;

use regex::bytes::Regex;
use std::{env, thread, time};
use storage::{BookRecord, Storage};

#[derive(Debug, Parser)]
#[command(name = "oreally")]
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
    #[command(about = "Queue a book using `pueue`")]
    Pueue {
        #[arg(long)]
        url: String,
        #[arg(long)]
        auth: Option<String>,
        #[arg(long)]
        folder: Option<String>,
    },
    #[command(about = "Start processing books queue")]
    Start {
        #[arg(long)]
        auth: Option<String>,
        #[arg(long)]
        folder: Option<String>,
    },
    #[command(about = "Initalises configuration")]
    Init,
    List,
}

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct BookRequest {
    book_id: String,
    title: String,
    auth: String,
    folder: String,
}

pub async fn run(cli: Cli) -> Result<()> {
    let storage = &Storage::default();
    cli.command.run(storage).await
}

impl Commands {
    pub async fn run(&self, storage: &Storage) -> Result<()> {
        match self {
            Self::Download { .. } => {
                let req = self.clone().try_into()?;
                download::run(req)
            }
            Self::Queue { url, .. } => {
                assert_readiness(storage)?;
                // Validate url
                parse_url(url)?;
                let mut record = BookRecord::new(url);
                record.insert(storage)?;
                Ok(())
            }
            Self::Pueue { .. } => {
                // Validate url
                let req: BookRequest = self.clone().try_into()?;
                pueue::run(&req).await?;
                Ok(())
            }
            Self::List => {
                assert_readiness(storage)?;
                let list = BookRecord::all(storage);
                BookRecord::table(&list)?.printstd();
                Ok(())
            }
            Self::Init { .. } => {
                assert_unreadiness(storage)?;
                storage.setup()?;
                Ok(())
            }
            Self::Start { .. } => {
                assert_readiness(storage)?;
                loop {
                    let list = BookRecord::all(storage);
                    BookRecord::table(&list)?.printstd();
                    for record in list {
                        let req = BookRequest::new_from_record(&record, self)?;
                        download::run(req)?;
                        record.delete(storage)?;
                    }
                    wait();
                }
            }
        }
    }
    pub fn auth(&self) -> Result<String> {
        match self {
            Self::Download { auth, .. } | Self::Start { auth, .. } => {
                let auth = get_arg_or_env(auth.clone(), "OREALLY_AUTH")
                    .ok_or("--auth argument or OREALLY_AUTH environment variable required")?;
                Ok(auth)
            }
            _ => panic!("invalid command"),
        }
    }
    pub fn folder(&self) -> Result<String> {
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
    fn new_from_record(record: &storage::BookRecord, cmd: &Commands) -> Result<Self> {
        let (epub_name, book_id) = parse_url(&record.url).unwrap();

        Ok(Self {
            book_id,
            title: epub_name,
            auth: cmd.auth()?,
            folder: cmd.folder()?,
        })
    }

    pub fn to_cmd(&self) -> String {
        let BookRequest {
            book_id,
            title,
            auth,
            folder,
        } = self;
        format!(
            "(docker run kirinnee/orly:latest login {book_id} {auth}) > \"{folder}/{title}.epub\"",
        )
    }
}

impl TryFrom<Commands> for BookRequest {
    type Error = Box<dyn error::Error>;

    fn try_from(value: Commands) -> Result<Self> {
        match value {
            Commands::Download { url, auth, folder } | Commands::Pueue { url, auth, folder } => {
                let (epub_name, book_id) =
                    parse_url(&url).unwrap_or_else(|_| panic!("invalid Url: {}", url));

                let auth = get_arg_or_env(auth, "OREALLY_AUTH")
                    .ok_or("--auth argument or OREALLY_AUTH environment variable required")?;
                let folder =
                    get_arg_or_env(folder, "OREALLY_FOLDER").unwrap_or_else(|| "~".to_string());
                Ok(BookRequest {
                    book_id,
                    title: epub_name,
                    auth,
                    folder,
                })
            }
            _ => panic!("invalid command"),
        }
    }
}

fn assert_readiness(storage: &Storage) -> Result<()> {
    if !storage.is_ready() {
        println!("Please run `oreilly init` to setup configuration");
        return Err("Not ready".into());
    }
    Ok(())
}
fn assert_unreadiness(storage: &Storage) -> Result<()> {
    if storage.is_ready() {
        println!("Oreilly is already initialized run `oreilly start` to start processing");
        return Err("Allready Initialized".into());
    }
    Ok(())
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

fn parse_url(url_str: &str) -> Result<(String, String)> {
    let re = Regex::new(r"learning.oreilly.com/library/view/(?P<name>.+)/(?P<id>\d+).*").unwrap();
    let captures = re
        .captures(url_str.as_bytes())
        .ok_or(format!("Invalid Url: {}", url_str))?;

    let id: String = std::str::from_utf8(
        captures
            .name("id")
            .ok_or("Invalid url: missing id")?
            .as_bytes(),
    )?
    .to_owned();
    let name: String = std::str::from_utf8(
        captures
            .name("name")
            .ok_or("Invalid url: missing name")?
            .as_bytes(),
    )?
    .to_owned();
    Ok((name, id))
}

#[cfg(test)]
mod test {
    use super::*;

    static URL: &str = "https://learning.oreilly.com/library/view/learn-postgresql/9781838985288";

    #[tokio::test]
    async fn commands_run_queue() {
        storage::test::around(async {
            |storage| {
                let url = URL.to_string();
                Commands::Queue { url: url.clone() }
                    .run(storage)
                    .await
                    .unwrap();
                let list = BookRecord::all(storage);
                assert_eq!(list.len(), 1);
                assert_eq!(list[0].url, url);
            }
        })
        .await
    }
    #[tokio::test]
    async fn commands_run_init() {
        let storage = Storage::new(Some("tmp/fdjkfd.db".to_string()));
        Commands::Init.run(&storage).await.unwrap();
        assert_readiness(&storage).unwrap();
        storage.drop().unwrap();
    }

    #[test]
    fn commands_auth() {
        assert_eq!(
            Commands::Start {
                auth: Some("auth".to_string()),
                folder: Some("folder".to_string()),
            }
            .auth()
            .unwrap(),
            "auth"
        );
        assert_eq!(
            Commands::Download {
                url: "tets".to_string(),
                auth: Some("auth".to_string()),
                folder: Some("folder".to_string()),
            }
            .auth()
            .unwrap(),
            "auth".to_string()
        );
    }
    #[test]
    fn commands_folder() {
        assert_eq!(
            Commands::Download {
                url: "tets".to_string(),
                auth: Some("auth".to_string()),
                folder: Some("folder".to_string()),
            }
            .folder()
            .unwrap(),
            "folder".to_string()
        );
    }

    #[test]
    fn book_request_new_from_record() {
        let r = BookRequest::new_from_record(
            &BookRecord::new(URL),
            &Commands::Start {
                auth: Some("auth".to_string()),
                folder: Some("folder".to_string()),
            },
        )
        .unwrap();

        assert_eq!(r.book_id, "9781838985288");
        assert_eq!(r.title, "learn-postgresql");
    }
    #[test]
    fn command_into_book_request() {
        let cmd = Commands::Download {
            url: URL.to_string(),
            auth: Some("auth".to_string()),
            folder: Some("folder".to_string()),
        };
        let r = BookRequest::try_from(cmd).unwrap();
        assert_eq!(r.book_id, "9781838985288");
        assert_eq!(r.title, "learn-postgresql");
    }

    #[test]
    fn test_parse_url() {
        let url_str = URL;
        let (title, id) = parse_url(url_str).unwrap();
        assert_eq!(title, "learn-postgresql");
        assert_eq!(id, "9781838985288");
        let url_str = format!("{URL}/fdfd");
        let (title, id) = parse_url(&url_str).unwrap();
        assert_eq!(title, "learn-postgresql");
        assert_eq!(id, "9781838985288");
    }

    #[test]
    fn get_arg_or_env_panics() {
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
