use std::{fs::File, io::Read, path::PathBuf};

use anyhow::{Context, Ok};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short = 'c', long = "config")]
    pub config_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub aptos_base_url: String,
    pub contract_address: String,
    pub decibel_url: String,
    pub server_config: ServerConfig,
    pub jwt_config: JWTConfig,
    pub db_config: DbConfig,
    pub turnkey_config: TurnkeyConfig,
    pub bot_config: BotConfig,
    pub admin_config: AdminConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTConfig {
    pub secret: String,
    pub expires_in: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub url: String,
    #[serde(default = "DbConfig::default_db_pool_size")]
    pub pool_size: u32,
}

impl DbConfig {
    pub const fn default_db_pool_size() -> u32 {
        10
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnkeyConfig {
    pub organization_id: String,
    pub api_private_key: String,
    pub api_public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub token: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    pub sponsor_private_key: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Config> {
        let cli = Cli::parse();
        let path_buf = cli.config_path.map(PathBuf::from);
        let path: PathBuf = path_buf.unwrap_or_else(|| PathBuf::from("config.yaml"));

        let mut file = File::open(&path).context("Failed to open the file path")?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .context("Failed to read the file path")?;

        let config =
            serde_yaml::from_str::<Config>(&contents).context("Failed to parse yaml file")?;

        Ok(config)
    }
}
