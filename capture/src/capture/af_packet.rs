//! AF_PACKET capture using pnet for cross-platform compatibility
//!
//! This module provides high-performance packet capture using pnet's
//! datalink layer, which uses AF_PACKET on Linux.

use anyhow::{Context, Result, bail};
use crossbeam_channel::{Sender, bounded};
use pnet::datalink::{self, Channel, Config};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use super::frame::CapturedFrame;
use super::interface::NetworkInterface;
use crate::decode;

/// Capture statistics
#[derive(Debug, Default)]
pub struct CaptureStats {
    /// Total packets captured
    pub packets_captured: AtomicU64,
    /// Total bytes captured
    pub bytes_captured: AtomicU64,
    /// Packets dropped (if available)
    pub packets_dropped: AtomicU64,
    /// Parse errors
    pub parse_errors: AtomicU64,
}

impl CaptureStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> CaptureStatsSnapshot {
        CaptureStatsSnapshot {
            packets_captured: self.packets_captured.load(Ordering::Relaxed),
            bytes_captured: self.bytes_captured.load(Ordering::Relaxed),
            packets_dropped: self.packets_dropped.load(Ordering::Relaxed),
            parse_errors: self.parse_errors.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of capture statistics (non-atomic copy)
#[derive(Debug, Clone)]
pub struct CaptureStatsSnapshot {
    pub packets_captured: u64,
    pub bytes_captured: u64,
    pub packets_dropped: u64,
    pub parse_errors: u64,
}

/// AF_PACKET based capture
pub struct AfPacketCapture {
    interface: NetworkInterface,
    promiscuous: bool,
    snap_length: usize,
    stats: Arc<CaptureStats>,
    running: Arc<AtomicBool>,
}

impl AfPacketCapture {
    /// Create a new AF_PACKET capture instance
    pub fn new(interface_name: &str, promiscuous: bool, snap_length: usize) -> Result<Self> {
        let interface = NetworkInterface::by_name(interface_name)?;
        interface.validate_for_capture()?;

        Ok(Self {
            interface,
            promiscuous,
            snap_length,
            stats: Arc::new(CaptureStats::new()),
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Get the interface name
    pub fn interface_name(&self) -> &str {
        &self.interface.name
    }

    /// Get capture statistics
    pub fn stats(&self) -> Arc<CaptureStats> {
        Arc::clone(&self.stats)
    }

    /// Check if capture is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Stop capture
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Start capture loop, sending frames to the provided channel
    pub fn start(&self, frame_sender: Sender<CapturedFrame>) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            bail!("Capture already running on interface {}", self.interface.name);
        }

        // Set promiscuous mode if requested
        if self.promiscuous {
            if let Err(e) = self.interface.set_promiscuous(true) {
                warn!("Failed to set promiscuous mode: {}", e);
            }
        }

        // Create datalink channel
        let config = Config {
            read_timeout: Some(Duration::from_millis(100)),
            write_buffer_size: 0, // We don't write
            read_buffer_size: 65536,
            ..Default::default()
        };

        // Find the pnet interface
        let interfaces = datalink::interfaces();
        let pnet_interface = interfaces
            .into_iter()
            .find(|i| i.name == self.interface.name)
            .with_context(|| format!("Interface '{}' not found", self.interface.name))?;

        let (_, mut rx) = match datalink::channel(&pnet_interface, config) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => bail!("Unhandled channel type"),
            Err(e) => bail!("Failed to create datalink channel: {}", e),
        };

        info!(
            "Started capture on interface '{}' (promiscuous: {})",
            self.interface.name, self.promiscuous
        );

        let interface_name = self.interface.name.clone();
        let stats = Arc::clone(&self.stats);
        let running = Arc::clone(&self.running);

        // Capture loop
        while running.load(Ordering::SeqCst) {
            match rx.next() {
                Ok(packet) => {
                    let frame_size = packet.len() as u32;

                    // Update stats
                    stats.packets_captured.fetch_add(1, Ordering::Relaxed);
                    stats.bytes_captured.fetch_add(frame_size as u64, Ordering::Relaxed);

                    // Decode the frame
                    match decode::parse_frame(&interface_name, packet) {
                        Ok(frame) => {
                            // Send to channel (non-blocking)
                            if let Err(e) = frame_sender.try_send(frame) {
                                debug!("Channel full, dropping frame: {}", e);
                            }
                        }
                        Err(e) => {
                            stats.parse_errors.fetch_add(1, Ordering::Relaxed);
                            debug!("Failed to parse frame: {}", e);
                        }
                    }
                }
                Err(e) => {
                    // Timeout is expected, other errors should be logged
                    let err_str = e.to_string().to_lowercase();
                    if !err_str.contains("timed out") && !err_str.contains("timeout") {
                        error!("Error receiving packet: {}", e);
                    }
                }
            }
        }

        // Cleanup: disable promiscuous mode
        if self.promiscuous {
            if let Err(e) = self.interface.set_promiscuous(false) {
                warn!("Failed to disable promiscuous mode: {}", e);
            }
        }

        info!("Capture stopped on interface '{}'", self.interface.name);
        Ok(())
    }

    /// Start capture in a new thread
    pub fn start_threaded(self: Arc<Self>, buffer_size: usize) -> Result<(std::thread::JoinHandle<()>, crossbeam_channel::Receiver<CapturedFrame>)> {
        let (tx, rx) = bounded(buffer_size);

        let capture = Arc::clone(&self);
        let handle = std::thread::spawn(move || {
            if let Err(e) = capture.start(tx) {
                error!("Capture thread error: {}", e);
            }
        });

        Ok((handle, rx))
    }
}

impl Drop for AfPacketCapture {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Multi-interface capture manager
pub struct MultiCapture {
    captures: Vec<Arc<AfPacketCapture>>,
    running: Arc<AtomicBool>,
}

impl MultiCapture {
    pub fn new() -> Self {
        Self {
            captures: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Add an interface to capture
    pub fn add_interface(&mut self, name: &str, promiscuous: bool, snap_length: usize) -> Result<()> {
        let capture = AfPacketCapture::new(name, promiscuous, snap_length)?;
        self.captures.push(Arc::new(capture));
        Ok(())
    }

    /// Start all captures
    pub fn start_all(&self, buffer_size: usize) -> Result<(Vec<std::thread::JoinHandle<()>>, crossbeam_channel::Receiver<CapturedFrame>)> {
        if self.captures.is_empty() {
            bail!("No interfaces configured for capture");
        }

        self.running.store(true, Ordering::SeqCst);

        // Create a single channel for all captures
        let (tx, rx) = bounded(buffer_size);
        let mut handles = Vec::new();

        for capture in &self.captures {
            let cap = Arc::clone(capture);
            let sender = tx.clone();

            let handle = std::thread::spawn(move || {
                if let Err(e) = cap.start(sender) {
                    error!("Capture error on {}: {}", cap.interface_name(), e);
                }
            });

            handles.push(handle);
        }

        Ok((handles, rx))
    }

    /// Stop all captures
    pub fn stop_all(&self) {
        self.running.store(false, Ordering::SeqCst);
        for capture in &self.captures {
            capture.stop();
        }
    }

    /// Get combined statistics from all captures
    pub fn combined_stats(&self) -> CaptureStatsSnapshot {
        let mut combined = CaptureStatsSnapshot {
            packets_captured: 0,
            bytes_captured: 0,
            packets_dropped: 0,
            parse_errors: 0,
        };

        for capture in &self.captures {
            let stats = capture.stats().snapshot();
            combined.packets_captured += stats.packets_captured;
            combined.bytes_captured += stats.bytes_captured;
            combined.packets_dropped += stats.packets_dropped;
            combined.parse_errors += stats.parse_errors;
        }

        combined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_stats() {
        let stats = CaptureStats::new();
        stats.packets_captured.fetch_add(100, Ordering::Relaxed);
        stats.bytes_captured.fetch_add(5000, Ordering::Relaxed);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.packets_captured, 100);
        assert_eq!(snapshot.bytes_captured, 5000);
    }

    #[test]
    fn test_multi_capture_empty() {
        let capture = MultiCapture::new();
        assert!(capture.start_all(1000).is_err());
    }
}
