//! Configuration module for NetSentinel Aggregator

use serde::Deserialize;
use std::path::Path;
use anyhow::{Context, Result};

/// Main configuration structure
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub redis: RedisConfig,
    pub database: DatabaseConfig,
    pub aggregation: AggregationConfig,
    #[serde(default)]
    pub events: EventsConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
}

/// Redis configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL
    #[serde(default = "default_redis_url")]
    pub url: String,

    /// Stream name to consume from
    #[serde(default = "default_stream_name")]
    pub stream_name: String,

    /// Consumer group name
    #[serde(default = "default_consumer_group")]
    pub consumer_group: String,

    /// Consumer name
    #[serde(default = "default_consumer_name")]
    pub consumer_name: String,

    /// Batch size for reading from stream
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Block timeout when reading (milliseconds)
    #[serde(default = "default_block_timeout", alias = "block_timeout_ms")]
    pub block_ms: u64,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,

    /// Connection pool size
    #[serde(default = "default_pool_size", alias = "pool_size")]
    pub max_connections: u32,

    /// Connection timeout (seconds)
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
}

/// Aggregation configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AggregationConfig {
    /// Persist interval for devices/flows (seconds)
    #[serde(default = "default_persist_interval", alias = "persist_interval")]
    pub persist_interval_secs: u64,

    /// Metrics bucket size
    #[serde(default = "default_metrics_bucket")]
    pub metrics_bucket: String,

    /// Device inactivity timeout (seconds)
    #[serde(default = "default_inactivity_timeout")]
    pub inactivity_timeout: u64,

    /// Flow timeout (seconds)
    #[serde(default = "default_flow_timeout")]
    pub flow_timeout: u64,
}

/// Events configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EventsConfig {
    /// Redis channel for real-time events
    #[serde(default = "default_events_channel")]
    pub channel: String,

    /// Publish new device events
    #[serde(default = "default_true")]
    pub publish_new_devices: bool,

    /// Publish new flow events
    #[serde(default = "default_true")]
    pub publish_new_flows: bool,

    /// Publish threshold alerts
    #[serde(default)]
    pub publish_alerts: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,

    #[serde(default)]
    pub file: Option<String>,

    #[serde(default = "default_true")]
    pub stdout: bool,

    #[serde(default = "default_log_format")]
    pub format: String,
}

/// Metrics configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MetricsConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_metrics_port")]
    pub port: u16,

    #[serde(default = "default_metrics_path")]
    pub path: String,
}

// Default value functions
fn default_redis_url() -> String { "redis://127.0.0.1:6379".to_string() }
fn default_stream_name() -> String { "netsentinel:frames".to_string() }
fn default_consumer_group() -> String { "aggregator".to_string() }
fn default_consumer_name() -> String { "aggregator-1".to_string() }
fn default_batch_size() -> usize { 100 }
fn default_block_timeout() -> u64 { 1000 }
fn default_pool_size() -> u32 { 10 }
fn default_connect_timeout() -> u64 { 30 }
fn default_persist_interval() -> u64 { 60 }
fn default_metrics_bucket() -> String { "1 minute".to_string() }
fn default_inactivity_timeout() -> u64 { 300 }
fn default_flow_timeout() -> u64 { 120 }
fn default_events_channel() -> String { "netsentinel:events".to_string() }
fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "pretty".to_string() }
fn default_true() -> bool { true }
fn default_metrics_port() -> u16 { 9101 }
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
        if self.database.max_connections < 1 {
            anyhow::bail!("Database max_connections must be at least 1");
        }

        if self.aggregation.persist_interval_secs < 1 {
            anyhow::bail!("Persist interval must be at least 1 second");
        }

        Ok(())
    }
}
