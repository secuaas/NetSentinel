//! Configuration module for NetSentinel Capture

use serde::Deserialize;
use std::path::Path;
use anyhow::{Context, Result};

/// Main configuration structure
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub capture: CaptureConfig,
    pub redis: RedisConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
}

/// Capture settings
#[derive(Debug, Clone, Deserialize)]
pub struct CaptureConfig {
    /// Capture mode: "mirror" or "bypass"
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Ring buffer size (number of frames)
    #[serde(default = "default_ring_buffer_size")]
    pub ring_buffer_size: usize,

    /// Maximum frame size to capture
    #[serde(default = "default_snap_length")]
    pub snap_length: usize,

    /// Flush interval in milliseconds
    #[serde(default = "default_flush_interval")]
    pub flush_interval_ms: u64,

    /// Batch size for Redis writes
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Network interfaces to monitor
    pub interfaces: Vec<InterfaceConfig>,
}

/// Interface configuration
#[derive(Debug, Clone, Deserialize)]
pub struct InterfaceConfig {
    pub name: String,
    #[serde(default = "default_true")]
    pub promiscuous: bool,
    #[serde(default)]
    pub description: Option<String>,
}

/// Redis configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL
    #[serde(default = "default_redis_url")]
    pub url: String,

    /// Stream name for captured frames
    #[serde(default = "default_stream_name")]
    pub stream_name: String,

    /// Maximum stream length
    #[serde(default = "default_max_stream_length")]
    pub max_stream_length: usize,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log file path
    #[serde(default)]
    pub file: Option<String>,

    /// Log to stdout
    #[serde(default = "default_true")]
    pub stdout: bool,

    /// Log format: "json" or "pretty"
    #[serde(default = "default_log_format")]
    pub format: String,
}

/// Metrics configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MetricsConfig {
    /// Enable metrics endpoint
    #[serde(default)]
    pub enabled: bool,

    /// Metrics port
    #[serde(default = "default_metrics_port")]
    pub port: u16,

    /// Metrics path
    #[serde(default = "default_metrics_path")]
    pub path: String,
}

// Default value functions
fn default_mode() -> String { "mirror".to_string() }
fn default_ring_buffer_size() -> usize { 8192 }
fn default_snap_length() -> usize { 1518 }
fn default_flush_interval() -> u64 { 100 }
fn default_batch_size() -> usize { 1000 }
fn default_redis_url() -> String { "redis://127.0.0.1:6379".to_string() }
fn default_stream_name() -> String { "netsentinel:frames".to_string() }
fn default_max_stream_length() -> usize { 100000 }
fn default_pool_size() -> usize { 4 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "pretty".to_string() }
fn default_true() -> bool { true }
fn default_metrics_port() -> u16 { 9100 }
fn default_metrics_path() -> String { "/metrics".to_string() }

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;

        toml::from_str(&content)
            .with_context(|| "Failed to parse configuration")
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate capture mode
        if self.capture.mode != "mirror" && self.capture.mode != "bypass" {
            anyhow::bail!("Invalid capture mode: {}. Must be 'mirror' or 'bypass'", self.capture.mode);
        }

        // Validate at least one interface
        if self.capture.interfaces.is_empty() {
            anyhow::bail!("At least one capture interface must be configured");
        }

        // Validate interface names
        for iface in &self.capture.interfaces {
            if iface.name.is_empty() {
                anyhow::bail!("Interface name cannot be empty");
            }
        }

        // Validate ring buffer size
        if self.capture.ring_buffer_size < 64 {
            anyhow::bail!("Ring buffer size must be at least 64");
        }

        // Validate snap length
        if self.capture.snap_length < 64 || self.capture.snap_length > 65535 {
            anyhow::bail!("Snap length must be between 64 and 65535");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_content = r#"
[capture]
mode = "mirror"
ring_buffer_size = 4096
snap_length = 1518
flush_interval_ms = 100
batch_size = 500

[[capture.interfaces]]
name = "eth0"
promiscuous = true

[redis]
url = "redis://localhost:6379"
stream_name = "test:frames"
max_stream_length = 50000

[logging]
level = "debug"
stdout = true
format = "pretty"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.capture.mode, "mirror");
        assert_eq!(config.capture.ring_buffer_size, 4096);
        assert_eq!(config.capture.interfaces.len(), 1);
        assert_eq!(config.capture.interfaces[0].name, "eth0");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_mode() {
        let toml_content = r#"
[capture]
mode = "invalid"

[[capture.interfaces]]
name = "eth0"

[redis]
url = "redis://localhost:6379"

[logging]
level = "info"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(config.validate().is_err());
    }
}
