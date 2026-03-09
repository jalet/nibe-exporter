use crate::myuplink::error::MyUplinkError;
use clap::Parser;
use std::path::PathBuf;

/// Configuration for nibe-exporter.
#[derive(Debug, Clone, Parser)]
#[command(
    name = "nibe-exporter",
    about = "Prometheus exporter for NIBE heat pumps via myUplink API",
    version
)]
pub struct Config {
    /// myUplink client ID (env: `NIBE_CLIENT_ID`).
    #[arg(long, env = "NIBE_CLIENT_ID", value_name = "ID")]
    pub client_id: Option<String>,

    /// myUplink client secret (env: `NIBE_CLIENT_SECRET`).
    #[arg(long, env = "NIBE_CLIENT_SECRET", value_name = "SECRET")]
    pub client_secret: Option<String>,

    /// myUplink API version: `v2` or `v3` (default: `v2`).
    #[arg(long, env = "NIBE_API_VERSION", default_value = "v2")]
    pub api_version: String,

    /// Device ID to export (env: `NIBE_DEVICE_ID`).
    #[arg(long, env = "NIBE_DEVICE_ID")]
    pub device_id: Option<String>,

    /// Poll interval in seconds (default: 60).
    #[arg(long, env = "NIBE_POLL_INTERVAL", default_value = "60")]
    pub poll_interval: u64,

    /// Listen address (default: 0.0.0.0:9090).
    #[arg(long, env = "NIBE_LISTEN_ADDR", default_value = "0.0.0.0:9090")]
    pub listen_addr: String,

    /// Log level: trace, debug, info, warn, error (default: info).
    #[arg(long, env = "NIBE_LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    /// Enable JSON logging.
    #[arg(long, env = "NIBE_LOG_JSON")]
    pub log_json: bool,

    /// Path to metrics mapping file (optional).
    #[arg(long, env = "NIBE_METRICS_MAPPING_FILE")]
    pub metrics_mapping_file: Option<PathBuf>,

    /// Path to client secret file (if using file-based secrets).
    #[arg(long, env = "NIBE_CLIENT_SECRET_FILE")]
    pub client_secret_file: Option<PathBuf>,

    /// Path to client ID file (if using file-based secrets).
    #[arg(long, env = "NIBE_CLIENT_ID_FILE")]
    pub client_id_file: Option<PathBuf>,
}

impl Config {
    /// Load configuration from CLI args and environment variables.
    #[must_use]
    pub fn load() -> Self {
        Self::parse()
    }

    /// Validate and resolve configuration.
    ///
    /// # Errors
    ///
    /// Returns `MyUplinkError` if:
    /// - Required fields (`client_id`, `client_secret`) are missing
    /// - API version is not `v2` or `v3`
    /// - Poll interval is zero
    /// - Secret files cannot be read
    pub fn validate(&mut self) -> Result<(), MyUplinkError> {
        // Resolve secrets
        if self.client_id.is_none() {
            if let Some(ref path) = self.client_id_file {
                self.client_id = Some(resolve_secret(path)?);
            }
        }

        if self.client_secret.is_none() {
            if let Some(ref path) = self.client_secret_file {
                self.client_secret = Some(resolve_secret(path)?);
            }
        }

        // Validate required fields
        if self.client_id.is_none() {
            return Err(MyUplinkError::ConfigError(
                "client_id is required (via --client-id or NIBE_CLIENT_ID)".to_string(),
            ));
        }

        if self.client_secret.is_none() {
            return Err(MyUplinkError::ConfigError(
                "client_secret is required (via --client-secret or NIBE_CLIENT_SECRET)".to_string(),
            ));
        }

        // Validate API version
        if self.api_version != "v2" && self.api_version != "v3" {
            return Err(MyUplinkError::InvalidApiVersion(self.api_version.clone()));
        }

        // Validate poll interval
        if self.poll_interval == 0 {
            return Err(MyUplinkError::ConfigError(
                "poll_interval must be > 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get client ID (assumes validation has been called).
    ///
    /// # Panics
    ///
    /// Panics if called before `validate()` or if validation was not called.
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn client_id(&self) -> &str {
        self.client_id.as_ref().expect("client_id not set")
    }

    /// Get client secret (assumes validation has been called).
    ///
    /// # Panics
    ///
    /// Panics if called before `validate()` or if validation was not called.
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn client_secret(&self) -> &str {
        self.client_secret.as_ref().expect("client_secret not set")
    }

    /// Get device ID if specified.
    #[must_use]
    pub fn device_id(&self) -> Option<&str> {
        self.device_id.as_deref()
    }
}

/// Resolve a secret from a file (e.g., for Kubernetes secrets mounted as files).
///
/// # Errors
///
/// Returns `MyUplinkError::ConfigError` if the file cannot be read.
fn resolve_secret(path: &std::path::Path) -> Result<String, MyUplinkError> {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .map_err(|e| MyUplinkError::ConfigError(format!("Failed to read secret file: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        // Note: This test requires setting NIBE_CLIENT_ID and NIBE_CLIENT_SECRET env vars
        // Just verify the structure can be parsed
        let defaults = Config {
            client_id: Some("test_id".to_string()),
            client_secret: Some("test_secret".to_string()),
            api_version: "v2".to_string(),
            device_id: None,
            poll_interval: 60,
            listen_addr: "0.0.0.0:9090".to_string(),
            log_level: "info".to_string(),
            log_json: false,
            metrics_mapping_file: None,
            client_secret_file: None,
            client_id_file: None,
        };

        assert_eq!(defaults.client_id(), "test_id");
        assert_eq!(defaults.client_secret(), "test_secret");
        assert_eq!(defaults.api_version, "v2");
        assert_eq!(defaults.poll_interval, 60);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config {
            client_id: Some("id".to_string()),
            client_secret: Some("secret".to_string()),
            api_version: "v2".to_string(),
            device_id: None,
            poll_interval: 60,
            listen_addr: "0.0.0.0:9090".to_string(),
            log_level: "info".to_string(),
            log_json: false,
            metrics_mapping_file: None,
            client_secret_file: None,
            client_id_file: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_missing_client_id() {
        let mut config = Config {
            client_id: None,
            client_secret: Some("secret".to_string()),
            api_version: "v2".to_string(),
            device_id: None,
            poll_interval: 60,
            listen_addr: "0.0.0.0:9090".to_string(),
            log_level: "info".to_string(),
            log_json: false,
            metrics_mapping_file: None,
            client_secret_file: None,
            client_id_file: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_api_version() {
        let mut config = Config {
            client_id: Some("id".to_string()),
            client_secret: Some("secret".to_string()),
            api_version: "v99".to_string(),
            device_id: None,
            poll_interval: 60,
            listen_addr: "0.0.0.0:9090".to_string(),
            log_level: "info".to_string(),
            log_json: false,
            metrics_mapping_file: None,
            client_secret_file: None,
            client_id_file: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_zero_poll_interval() {
        let mut config = Config {
            client_id: Some("id".to_string()),
            client_secret: Some("secret".to_string()),
            api_version: "v2".to_string(),
            device_id: None,
            poll_interval: 0,
            listen_addr: "0.0.0.0:9090".to_string(),
            log_level: "info".to_string(),
            log_json: false,
            metrics_mapping_file: None,
            client_secret_file: None,
            client_id_file: None,
        };

        assert!(config.validate().is_err());
    }
}
