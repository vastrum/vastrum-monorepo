use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub chain: ChainConfig,
    pub sync: SyncConfig,
}

#[derive(Deserialize, Clone)]
pub struct ServerConfig {
    pub http_port: u16,
    pub ssh_port: u16,
    pub data_dir: PathBuf,
    pub ssh_host_key_path: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct ChainConfig {
    pub relay_key_path: PathBuf,
    pub gitter_domain: String,
}

#[derive(Deserialize, Clone)]
pub struct SyncConfig {
    pub poll_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                http_port: 8080,
                ssh_port: 2222,
                data_dir: PathBuf::from("./relay-data/repos"),
                ssh_host_key_path: PathBuf::from("./relay-data/ssh_host_ed25519_key"),
            },
            chain: ChainConfig {
                relay_key_path: PathBuf::from("../relay.key"),
                gitter_domain: "gitter".to_string(),
            },
            sync: SyncConfig {
                poll_interval_secs: 30,
            },
        }
    }
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
