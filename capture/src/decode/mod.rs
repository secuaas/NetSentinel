//! Frame decoding module
//!
//! Handles parsing of Ethernet frames including VLAN tags,
//! IPv4 headers, and TCP/UDP ports.

pub mod ethernet;
pub mod vlan;
pub mod ipv4;
pub mod transport;

use anyhow::Result;
use crate::capture::frame::CapturedFrame;

pub use ethernet::parse_ethernet;
pub use vlan::{parse_vlan, parse_qinq};
pub use ipv4::parse_ipv4;
pub use transport::parse_transport;

/// Parse a complete frame from raw bytes
pub fn parse_frame(interface: &str, data: &[u8]) -> Result<CapturedFrame> {
    ethernet::parse_frame(interface, data)
}
