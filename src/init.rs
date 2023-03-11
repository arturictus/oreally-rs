use crate::Commands;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::{error, fs, fs::File, path::Path};

struct Config {
    config_folder: String,
    books_folder: String,
    username: String,
    password: String,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ConfigBuilder {
    config_folder: Option<String>,
    books_folder: Option<String>,
    username: Option<String>,
    password: Option<String>,
}
pub fn run(cmd: Commands) -> Result<(), Box<dyn error::Error>> {
    if let Commands::Init {
        config_folder,
        books_folder,
        username,
        password,
    } = cmd
    {
        let config = ConfigBuilder {
            config_folder,
            books_folder,
            username,
            password,
        };
        let yaml = serde_yaml::to_string(&config)?;

        write_file("tmp/config.yml".to_string(), yaml)?;
        Ok(())
    } else {
        todo!()
    }
}

fn write_file(file: String, content: String) -> Result<(), Box<dyn error::Error>> {
    let folder = Path::new(&file).parent().unwrap();
    fs::create_dir_all(folder).unwrap_or_else(|_| {
        panic!("unable to create folder {folder:?}");
    });
    let mut file = File::create(file)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
