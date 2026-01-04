use crate::error::{RdtError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub reddit: RedditConfig,
    #[serde(default)]
    pub aws: AwsConfig,
    #[serde(skip)]
    config_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RedditConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AwsConfig {
    pub region: Option<String>,
    pub bedrock_model_id: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_path = config_dir.join("config.toml");

        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content).map_err(|e| RdtError::Config(e.to_string()))?
        } else {
            Config::default()
        };

        config.config_dir = config_dir;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = &self.config_dir;
        fs::create_dir_all(config_dir)?;

        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self).map_err(|e| RdtError::Config(e.to_string()))?;
        fs::write(&config_path, content)?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&config_path, perms)?;
        }

        Ok(())
    }

    pub fn config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join("rdt"))
            .ok_or_else(|| RdtError::Config("Could not find config directory".to_string()))
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    pub fn has_credentials(&self) -> bool {
        self.reddit.access_token.is_some() || self.reddit.client_id.is_some()
    }

    pub fn clear_credentials(&mut self) -> Result<()> {
        self.reddit.access_token = None;
        self.reddit.refresh_token = None;
        self.save()
    }

    pub fn user_agent(&self) -> String {
        self.reddit
            .user_agent
            .clone()
            .unwrap_or_else(|| format!("rdt/{} (Rust CLI)", env!("CARGO_PKG_VERSION")))
    }

    pub fn bedrock_model_id(&self) -> String {
        self.aws
            .bedrock_model_id
            .clone()
            .unwrap_or_else(|| "anthropic.claude-3-haiku-20240307-v1:0".to_string())
    }
}
