//! NetSentinel Aggregator - Data Aggregation Service
//!
//! Consumes captured frames from Redis, aggregates them, and persists to PostgreSQL.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use netsentinel_aggregator::config::Config;
use netsentinel_aggregator::pipeline::Pipeline;

/// NetSentinel Aggregator Service
#[derive(Parser, Debug)]
#[command(name = "netsentinel-aggregator")]
#[command(author = "SecuAAS")]
#[command(version)]
#[command(about = "Aggregates network data and persists to database", long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "/opt/netsentinel/config/aggregator.toml")]
    config: PathBuf,

    /// Run in debug mode (verbose logging)
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = Config::from_file(&args.config)
        .with_context(|| format!("Failed to load config from {:?}", args.config))?;

    config.validate()?;

    // Setup logging
    setup_logging(&config, args.debug)?;

    info!("NetSentinel Aggregator starting...");
    info!("Redis: {}", config.redis.url);
    info!("Database: {}", config.database.url);

    // Create and start the pipeline
    let pipeline = Arc::new(
        Pipeline::new(config)
            .await
            .with_context(|| "Failed to initialize pipeline")?
    );

    // Setup signal handling
    let pipeline_shutdown = Arc::clone(&pipeline);
    ctrlc::set_handler(move || {
        info!("Received shutdown signal");
        pipeline_shutdown.shutdown();
    })
    .context("Failed to set Ctrl+C handler")?;

    // Run the pipeline
    pipeline.run().await?;

    info!("NetSentinel Aggregator stopped");
    Ok(())
}

/// Setup logging based on configuration
fn setup_logging(config: &Config, debug: bool) -> Result<()> {
    let level = if debug {
        Level::DEBUG
    } else {
        match config.logging.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" | "warning" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    };

    let filter = EnvFilter::from_default_env()
        .add_directive(format!("netsentinel_aggregator={}", level).parse().unwrap())
        .add_directive("sqlx=warn".parse().unwrap())
        .add_directive("redis=warn".parse().unwrap());

    let subscriber = tracing_subscriber::registry().with(filter);

    if config.logging.format == "json" {
        subscriber.with(fmt::layer().json()).init();
    } else {
        subscriber.with(fmt::layer().with_target(true)).init();
    }

    Ok(())
}
