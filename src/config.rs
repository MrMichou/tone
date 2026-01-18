//! Configuration Module
//!
//! Handles persistent configuration for tone.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Persistent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Last used endpoint
    #[serde(default)]
    pub endpoint: Option<String>,

    /// Last used username
    #[serde(default)]
    pub username: Option<String>,
}

impl Config {
    /// Load configuration from disk
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Some(path) = path {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path().ok_or_else(|| anyhow::anyhow!("No config directory"))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, &content)?;

        // Set restrictive permissions (0600) on Unix systems to protect config
        #[cfg(unix)]
        {
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&path, permissions)?;
        }

        Ok(())
    }

    /// Get the config file path
    fn config_path() -> Option<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            return Some(config_dir.join("tone").join("config.json"));
        }
        if let Some(home) = dirs::home_dir() {
            return Some(home.join(".tone").join("config.json"));
        }
        None
    }

    /// Get effective endpoint (from config or env)
    pub fn effective_endpoint(&self) -> String {
        std::env::var("ONE_XMLRPC")
            .ok()
            .or_else(|| self.endpoint.clone())
            .unwrap_or_else(|| "http://localhost:2633/RPC2".to_string())
    }

    /// Set endpoint and save
    pub fn set_endpoint(&mut self, endpoint: &str) -> Result<()> {
        self.endpoint = Some(endpoint.to_string());
        self.save()
    }
}
