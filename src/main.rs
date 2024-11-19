use ::log::info;
use anyhow::anyhow;
use clap::Parser;
use server::http_server::HttpServer;

use signal_handle::SignalHandler;
use tokio::{main, signal::unix::SignalKind};
mod config;
mod layer;
mod log;
mod server;
mod signal_handle;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to config file, defaults to ~/.peregrein.config
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
}

#[main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Get config path from CLI or use default
    let config_path = if let Some(path) = cli.config {
        path
    } else {
        home::home_dir()
            .ok_or(anyhow!("Cannot find home directory"))?
            .join(".peregrein.config")
    };

    let config =
        config::Config::from_yaml_file(config_path).map_err(|err| anyhow!(err.to_string()))?;

    log::init_logger(config.log_config)?;

    let mut handlers = Vec::new();
    for server_config in config.servers {
        let handler = HttpServer::new(server_config).run().await?;
        handlers.push(handler);
    }
    handle_signals().await?;
    Ok(())
}

async fn handle_signals() -> anyhow::Result<()> {
    let mut handler = SignalHandler::new();

    handler
        .handle_signal(SignalKind::terminate(), || {
            info!("Received SIGTERM signal, shutting down...");
            std::process::exit(0);
        })?
        .handle_signal(SignalKind::interrupt(), || {
            info!("Received SIGINT signal, shutting down...");
            std::process::exit(0);
        })?
        .handle_signal(SignalKind::hangup(), || {
            info!("Received SIGHUP signal, reloading config...");
            // TODO: graceful upgrade
        })?;

    handler.run().await
}
