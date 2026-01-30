//! NetSentinel Capture Module
//!
//! Passive network packet capture using AF_PACKET for high-performance
//! zero-copy frame capture on Linux systems.

pub mod capture;
pub mod config;
pub mod decode;
pub mod output;

pub use config::Config;
