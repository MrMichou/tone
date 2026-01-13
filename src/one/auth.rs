//! OpenNebula Authentication
//!
//! Handles authentication using OpenNebula credentials from:
//! - Environment variables (ONE_AUTH, ONE_XMLRPC)
//! - Config file (~/.one/one_auth)
//!
//! Security features:
//! - Credentials are zeroized on drop to prevent memory leaks
//! - Custom Debug implementation redacts sensitive data
//! - File permissions are validated before reading credentials
//! - HTTPS is used by default for API endpoints

use anyhow::{Context, Result};
use std::path::PathBuf;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// OpenNebula credentials holder
///
/// This struct implements secure credential handling:
/// - Password is zeroized when the struct is dropped
/// - Debug output redacts the password
/// - Fields are private to prevent accidental exposure
#[derive(Clone, ZeroizeOnDrop)]
pub struct OneCredentials {
    #[zeroize(skip)] // Username is not sensitive
    username: String,
    password: String,
    #[zeroize(skip)] // Endpoint is not sensitive
    endpoint: String,
}

// Custom Debug implementation that redacts sensitive data
impl std::fmt::Debug for OneCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OneCredentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

impl OneCredentials {
    /// Create credentials from environment or config file
    pub fn new() -> Result<Self> {
        let auth_string = Self::get_auth_string()?;
        let endpoint = Self::get_endpoint();

        let (username, password) = Self::parse_auth_string(&auth_string)?;

        // Warn if using HTTP for non-localhost endpoints
        Self::warn_insecure_endpoint(&endpoint);

        Ok(Self {
            username,
            password,
            endpoint,
        })
    }

    /// Get the username (read-only access)
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Get the endpoint URL (read-only access)
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Set a custom endpoint
    pub fn set_endpoint(&mut self, endpoint: String) {
        Self::warn_insecure_endpoint(&endpoint);
        self.endpoint = endpoint;
    }

    /// Warn if using insecure HTTP for remote endpoints
    fn warn_insecure_endpoint(endpoint: &str) {
        if endpoint.starts_with("http://") {
            // Allow HTTP only for localhost/127.0.0.1
            let is_localhost = endpoint.contains("localhost") || endpoint.contains("127.0.0.1");
            if !is_localhost {
                tracing::warn!(
                    "Using insecure HTTP connection to remote host. \
                     Consider using HTTPS for production environments."
                );
            }
        }
    }

    /// Get the authentication string (username:password)
    fn get_auth_string() -> Result<String> {
        // Check ONE_AUTH environment variable first
        if let Ok(auth) = std::env::var("ONE_AUTH") {
            // ONE_AUTH can be a path to a file or the auth string directly
            let path = PathBuf::from(&auth);
            if path.exists() {
                Self::validate_file_permissions(&path)?;
                let mut content =
                    std::fs::read_to_string(&path).context("Failed to read ONE_AUTH file")?;
                let result = content.trim().to_string();
                // Zeroize the original content
                content.zeroize();
                return Ok(result);
            }
            return Ok(auth);
        }

        // Try default config file
        let auth_file = Self::get_default_auth_file();
        if auth_file.exists() {
            Self::validate_file_permissions(&auth_file)?;
            let mut content = std::fs::read_to_string(&auth_file)
                .context("Failed to read ~/.one/one_auth file")?;
            let result = content.trim().to_string();
            // Zeroize the original content
            content.zeroize();
            return Ok(result);
        }

        Err(anyhow::anyhow!(
            "No OpenNebula credentials found. Set ONE_AUTH environment variable or create ~/.one/one_auth"
        ))
    }

    /// Validate that credential file has secure permissions (Unix only)
    #[cfg(unix)]
    fn validate_file_permissions(path: &PathBuf) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let metadata = std::fs::metadata(path).context("Failed to read file metadata")?;
        let mode = metadata.permissions().mode();

        // Check if file is readable by group or others (bits 0o077)
        if mode & 0o077 != 0 {
            tracing::warn!(
                "Credential file {:?} has insecure permissions {:o}. \
                 Recommended: chmod 600 {:?}",
                path,
                mode & 0o777,
                path
            );
        }

        Ok(())
    }

    /// Validate file permissions (no-op on non-Unix systems)
    #[cfg(not(unix))]
    fn validate_file_permissions(_path: &PathBuf) -> Result<()> {
        Ok(())
    }

    /// Get the default auth file path
    fn get_default_auth_file() -> PathBuf {
        dirs::home_dir()
            .map(|h| h.join(".one").join("one_auth"))
            .unwrap_or_else(|| PathBuf::from(".one/one_auth"))
    }

    /// Get the XML-RPC endpoint URL
    /// Default is HTTPS for security
    fn get_endpoint() -> String {
        std::env::var("ONE_XMLRPC").unwrap_or_else(|_| "https://localhost:2633/RPC2".to_string())
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
    /// Note: The returned string contains sensitive data and should be
    /// zeroized after use if stored in a variable
    pub fn auth_string(&self) -> String {
        format!("{}:{}", self.username, self.password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_auth_string() {
        let (user, pass) = OneCredentials::parse_auth_string("testuser:testpass").unwrap();
        assert_eq!(user, "testuser");
        assert_eq!(pass, "testpass");
    }

    #[test]
    fn test_parse_auth_string_with_colon_in_password() {
        let (user, pass) = OneCredentials::parse_auth_string("testuser:test:pass:123").unwrap();
        assert_eq!(user, "testuser");
        assert_eq!(pass, "test:pass:123");
    }

    #[test]
    fn test_debug_redacts_password() {
        // This test verifies that Debug output doesn't contain the actual password
        let creds = OneCredentials {
            username: "testuser".to_string(),
            password: "supersecret".to_string(),
            endpoint: "https://localhost:2633/RPC2".to_string(),
        };
        let debug_output = format!("{:?}", creds);
        assert!(!debug_output.contains("supersecret"));
        assert!(debug_output.contains("[REDACTED]"));
        assert!(debug_output.contains("testuser"));
    }
}
