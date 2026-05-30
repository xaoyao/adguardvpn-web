use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub vpn: VpnConfig,
    pub auth: AuthConfig,
}

#[derive(Deserialize, Clone)]
pub struct ServerConfig {
    pub bind: String,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct VpnConfig {
    pub cli_path: String,
}

#[derive(Deserialize, Clone)]
pub struct AuthConfig {
    pub password: String,
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(Path::new(path))?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
