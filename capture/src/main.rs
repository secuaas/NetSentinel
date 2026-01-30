//! NetSentinel Capture - Passive Network Packet Capture
//!
//! High-performance packet capture for network monitoring and CMDB generation.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use netsentinel_capture::capture::{CapturedFrame, MultiCapture, print_interfaces};
use netsentinel_capture::config::Config;
use netsentinel_capture::output::RedisOutput;

/// NetSentinel Passive Network Capture
#[derive(Parser, Debug)]
#[command(name = "netsentinel-capture")]
#[command(author = "SecuAAS")]
#[command(version)]
#[command(about = "Passive network packet capture for CMDB generation", long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "/opt/netsentinel/config/capture.toml")]
    config: PathBuf,

    /// List available network interfaces
    #[arg(long)]
    list_interfaces: bool,

    /// Run in debug mode (verbose logging)
    #[arg(short, long)]
    debug: bool,

    /// Dry run - capture but don't send to Redis
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // List interfaces and exit if requested
    if args.list_interfaces {
        print_interfaces();
        return Ok(());
    }

    // Load configuration
    let config = Config::from_file(&args.config)
        .with_context(|| format!("Failed to load config from {:?}", args.config))?;

    config.validate()?;

    // Setup logging
    setup_logging(&config, args.debug)?;

    info!("NetSentinel Capture starting...");
    info!("Mode: {}", config.capture.mode);
    info!("Interfaces: {:?}", config.capture.interfaces.iter().map(|i| &i.name).collect::<Vec<_>>());

    // Create channel for frames
    let (frame_tx, frame_rx) = mpsc::channel::<CapturedFrame>(config.capture.ring_buffer_size);

    // Start Redis output (unless dry run)
    let redis_handle = if !args.dry_run {
        let redis_output = RedisOutput::new(config.redis.clone());
        let batch_size = config.capture.batch_size;
        let flush_interval = config.capture.flush_interval_ms;

        Some(tokio::spawn(async move {
            if let Err(e) = redis_output.run(frame_rx, batch_size, flush_interval).await {
                error!("Redis output error: {}", e);
            }
        }))
    } else {
        info!("Dry run mode - frames will not be sent to Redis");
        // Consume frames but don't do anything with them
        let mut rx = frame_rx;
        Some(tokio::spawn(async move {
            let mut count = 0u64;
            while rx.recv().await.is_some() {
                count += 1;
                if count % 10000 == 0 {
                    info!("Dry run: {} frames captured", count);
                }
            }
            info!("Dry run: Total {} frames captured", count);
        }))
    };

    // Setup capture on all interfaces
    let mut multi_capture = MultiCapture::new();
    for iface in &config.capture.interfaces {
        if let Err(e) = multi_capture.add_interface(
            &iface.name,
            iface.promiscuous,
            config.capture.snap_length,
        ) {
            error!("Failed to add interface '{}': {}", iface.name, e);
        }
    }

    // Start capture threads
    let (capture_handles, capture_rx): (Vec<std::thread::JoinHandle<()>>, crossbeam_channel::Receiver<CapturedFrame>) = multi_capture
        .start_all(config.capture.ring_buffer_size)
        .with_context(|| "Failed to start capture")?;

    info!("Capture started on {} interface(s)", capture_handles.len());

    // Bridge capture channel to frame_tx
    let bridge_handle = tokio::spawn(async move {
        while let Ok(frame) = capture_rx.recv() {
            if frame_tx.send(frame).await.is_err() {
                warn!("Frame channel closed");
                break;
            }
        }
    });

    // Setup signal handling
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = Arc::clone(&running);

    ctrlc::set_handler(move || {
        info!("Received shutdown signal");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    }).context("Failed to set Ctrl+C handler")?;

    // Wait for shutdown signal
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Cleanup
    info!("Shutting down...");
    multi_capture.stop_all();

    // Print final stats
    let stats = multi_capture.combined_stats();
    info!(
        "Final stats: packets={}, bytes={}, dropped={}, errors={}",
        stats.packets_captured,
        stats.bytes_captured,
        stats.packets_dropped,
        stats.parse_errors
    );

    // Wait for capture threads
    for handle in capture_handles {
        let _ = handle.join();
    }

    // Cancel bridge and redis tasks
    bridge_handle.abort();
    if let Some(h) = redis_handle {
        h.abort();
    }

    info!("NetSentinel Capture stopped");
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
        .add_directive(format!("netsentinel_capture={}", level).parse().unwrap())
        .add_directive("redis=warn".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap());

    let subscriber = tracing_subscriber::registry()
        .with(filter);

    if config.logging.format == "json" {
        subscriber
            .with(fmt::layer().json())
            .init();
    } else {
        subscriber
            .with(fmt::layer().with_target(true))
            .init();
    }

    Ok(())
}
