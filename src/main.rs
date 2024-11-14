use anyhow::anyhow;
use clap::Parser;
use server::http_server::HttpServer;
use tokio::main;
mod config;
mod layer;
mod log;
mod server;

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

    let server_config = config
        .servers
        .into_iter()
        .next()
        .ok_or(anyhow!("No server config found"))?;
    HttpServer::new(server_config).run().await?;

    Ok(())
}
