use std::path::PathBuf;

use anyhow::{bail, Context};
use log::warn;
use simplelog::{Config, ConfigBuilder, LevelFilter, SimpleLogger};

use crate::BookRequest;

use pueue_lib::settings::{configuration_directories, Settings};
// use pueue::client::cli::{CliArguments, ColorChoice, Shell, SubCommand};
use pueue::client::cli as pueue_cli;
use pueue::client::client::Client;

// std::result::Result<T, >
pub async fn run(req: &BookRequest) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = pueue_cli::SubCommand::Add {
        command: vec![req.to_cmd()],
        working_directory: None,
        escape: false,
        start_immediately: true,
        stashed: false,
        delay_until: None,
        group: None,
        dependencies: vec![],
        label: None,
        print_task_id: true,
    };

    // let mut config_path = configuration_directories();
    // config_path.push("pueue.toml");

    let opt = pueue_cli::CliArguments {
        cmd: Some(cmd),
        config: None,
        color: pueue_cli::ColorChoice::Auto,
        profile: None,
        verbose: 0,
    };
    // Init the logger and set the verbosity level depending on the `-v` flags.
    let level = match &opt.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };

    // Try to initialize the logger with the timezone set to the Local time of the machine.
    let mut builder = ConfigBuilder::new();
    let logger_config = match builder.set_time_offset_to_local() {
        Err(_) => {
            warn!("Failed to determine the local time of this machine. Fallback to UTC.");
            Config::default()
        }
        Ok(builder) => builder.build(),
    };

    SimpleLogger::init(level, logger_config).unwrap();

    // Try to read settings from the configuration file.
    let (mut settings, config_found) =
        Settings::read(&opt.config).context("Failed to read configuration.")?;

    // Load any requested profile.
    if let Some(profile) = &opt.profile {
        settings.load_profile(profile)?;
    }

    #[allow(deprecated)]
    if settings.daemon.groups.is_some() {
        println!(
            "Please delete the 'daemon.groups' section from your config file.
            Run `pueue -vv` to see where your config is located.\n\
            It is no longer used and groups can now only be edited via the commandline interface.\n\n\
            Attention: The first time the daemon is restarted this update, the amount of parallel tasks per group will be reset to 1!!"
        )
    }

    // Error if no configuration file can be found, as this is an indicator, that the daemon hasn't
    // been started yet.
    if !config_found {
        panic!("Couldn't find a configuration file. Did you start the daemon yet?");
    }

    // Create client to talk with the daemon and connect.
    let mut client = Client::new(settings, opt)
        .await
        .context("Failed to initialize client.")?;
    client.start().await?;

    Ok(())
}
