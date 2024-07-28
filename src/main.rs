use anyhow::anyhow;
use server::http_server::HttpServer;
use tokio::main;
mod config;
mod layer;
mod server;
#[main]
async fn main() -> anyhow::Result<()> {
    // Read the configuration file
    let home_path = home::home_dir().ok_or(anyhow!("Cannot find home directory"))?;
    let config_path = home_path.join(".peregrein.config");
    let config =
        config::Config::from_toml_file(config_path).map_err(|err| anyhow!(err.to_string()))?;

    let server_config = config.servers.into_iter().next().unwrap();

    HttpServer::new(server_config).run().await?;

    Ok(())
}
