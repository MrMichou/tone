//! OpenNebula Authentication
//!
//! Handles authentication using OpenNebula credentials from:
//! - Environment variables (ONE_AUTH, ONE_XMLRPC)
//! - Config file (~/.one/one_auth)

use anyhow::{Context, Result};
use std::path::PathBuf;

/// OpenNebula credentials holder
#[derive(Clone, Debug)]
pub struct OneCredentials {
    pub username: String,
    pub password: String,
    pub endpoint: String,
}

impl OneCredentials {
    /// Create credentials from environment or config file
    pub fn new() -> Result<Self> {
        let auth_string = Self::get_auth_string()?;
        let endpoint = Self::get_endpoint();

        let (username, password) = Self::parse_auth_string(&auth_string)?;

        Ok(Self {
            username,
            password,
            endpoint,
        })
    }

    /// Get the authentication string (username:password)
    fn get_auth_string() -> Result<String> {
        // Check ONE_AUTH environment variable first
        if let Ok(auth) = std::env::var("ONE_AUTH") {
            // ONE_AUTH can be a path to a file or the auth string directly
            let path = PathBuf::from(&auth);
            if path.exists() {
                return std::fs::read_to_string(&path)
                    .map(|s| s.trim().to_string())
                    .context("Failed to read ONE_AUTH file");
            }
            return Ok(auth);
        }

        // Try default config file
        let auth_file = Self::get_default_auth_file();
        if auth_file.exists() {
            return std::fs::read_to_string(&auth_file)
                .map(|s| s.trim().to_string())
                .context("Failed to read ~/.one/one_auth file");
        }

        Err(anyhow::anyhow!(
            "No OpenNebula credentials found. Set ONE_AUTH environment variable or create ~/.one/one_auth"
        ))
    }

    /// Get the default auth file path
    fn get_default_auth_file() -> PathBuf {
        dirs::home_dir()
            .map(|h| h.join(".one").join("one_auth"))
            .unwrap_or_else(|| PathBuf::from(".one/one_auth"))
    }

    /// Get the XML-RPC endpoint URL
    fn get_endpoint() -> String {
        std::env::var("ONE_XMLRPC").unwrap_or_else(|_| "http://localhost:2633/RPC2".to_string())
    }

    /// Parse auth string into username and password
    fn parse_auth_string(auth: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = auth.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid auth format. Expected 'username:password'"
            ));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Get the auth string for XML-RPC calls
    pub fn auth_string(&self) -> String {
        format!("{}:{}", self.username, self.password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_auth_string() {
        let (user, pass) = OneCredentials::parse_auth_string("admin:password123").unwrap();
        assert_eq!(user, "admin");
        assert_eq!(pass, "password123");
    }

    #[test]
    fn test_parse_auth_string_with_colon_in_password() {
        let (user, pass) = OneCredentials::parse_auth_string("admin:pass:word:123").unwrap();
        assert_eq!(user, "admin");
        assert_eq!(pass, "pass:word:123");
    }
}
