use clap::{Parser, Subcommand};
use tracing::info;
use std::{error::Error, time::Duration};

use crate::{tui, wifi};

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(propagate_version = true)]
#[command(arg_required_else_help = false)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Ui,
    Conf {
        #[command(subcommand)]
        conf_cmd: ConfCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ConfCommands {
    Wifi {
        #[command(subcommand)]
        wifi_cmd: WifiCommands,
    },
}

#[derive(Subcommand, Debug)]
enum WifiCommands {
    SetConnection {
        ssid: String,
        password: String,
    },
    CheckConnectivity,
    Up,
    Down,
}

pub async fn handle() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Ui) {
        Commands::Ui => {
            tui::run().await?;
        }
        Commands::Conf { conf_cmd } => match conf_cmd {
            ConfCommands::Wifi { wifi_cmd } => match wifi_cmd {
                WifiCommands::SetConnection { ssid, password } => {
                    wifi::set_connection(&ssid, &password).await?
                },
                WifiCommands::CheckConnectivity => {
                    let connectivity = wifi::check_connectivity().await?;
                    info!("{:?}", connectivity);
                },
                WifiCommands::Up => {
                    wifi::up_connection(Duration::from_secs(30)).await?
                },
                WifiCommands::Down => {
                    wifi::down_connection(Duration::from_secs(30)).await?
                }
            }
        },
    }

    Ok(())
}
