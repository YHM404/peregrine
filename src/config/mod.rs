use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Config {
    pub(crate) servers: HashSet<ServerConfig>,
}

impl Config {
    pub(crate) fn from_toml_file<P: AsRef<Path>>(
        file: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file)?;
        Ok(toml::from_str(&content)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) enum Protocol {
    Tcp,
    Http,
    Https,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct ServerConfig {
    pub(crate) name: String,
    pub(crate) port: u16,
    pub(crate) protocol: Protocol,
    pub(crate) backends: HashMap<String, BackendConfig>,
}

impl Hash for ServerConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct BackendConfig {
    // TODO: consider add TLS support
    pub(crate) host: String,
    pub(crate) port: u16,
    #[serde(default)]
    pub(crate) enable_h2c: bool,
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_config_from_toml_file() {
        let cfg_str = r#"[[servers]]
        name = "test"
        port = 8080
        protocol = "Http"
        
        [servers.backends.backend1]
        host = "localhost"
        port = 8081
        enable_h2c = true"#;

        let mut tmp_file = tempfile::Builder::new()
            .prefix("config")
            .suffix(".toml")
            .tempfile()
            .unwrap();
        tmp_file.write_all(cfg_str.as_bytes()).unwrap();

        let config = Config::from_toml_file(tmp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.servers.len(), 1);
        let server = config.servers.iter().next().unwrap();
        assert_eq!(server.name, "test");
        assert_eq!(server.port, 8080);
        assert_eq!(server.protocol, Protocol::Http);
        assert_eq!(server.backends.len(), 1);
        let backend = server.backends.get("backend1").unwrap();
        assert_eq!(backend.host, "localhost");
        assert_eq!(backend.port, 8081);
    }
}
