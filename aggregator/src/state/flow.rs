//! Flow state management

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::net::Ipv4Addr;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::MacAddr;

/// Unique key for a flow
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowKey {
    pub src_mac: MacAddr,
    pub dst_mac: MacAddr,
    pub src_ip: Option<Ipv4Addr>,
    pub dst_ip: Option<Ipv4Addr>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub vlan_id: Option<u16>,
    pub protocol: Option<u8>,
}

impl FlowKey {
    /// Create a string representation for logging
    pub fn to_display_string(&self) -> String {
        let src = if let Some(ip) = self.src_ip {
            if let Some(port) = self.src_port {
                format!("{}:{}", ip, port)
            } else {
                ip.to_string()
            }
        } else {
            self.src_mac.to_string()
        };

        let dst = if let Some(ip) = self.dst_ip {
            if let Some(port) = self.dst_port {
                format!("{}:{}", ip, port)
            } else {
                ip.to_string()
            }
        } else {
            self.dst_mac.to_string()
        };

        let proto = self.protocol.map(|p| match p {
            1 => "ICMP",
            6 => "TCP",
            17 => "UDP",
            _ => "OTHER",
        }).unwrap_or("L2");

        format!("{} -> {} [{}]", src, dst, proto)
    }
}

/// Flow state in memory
pub struct FlowState {
    /// Unique identifier
    pub id: Uuid,

    /// Flow key (tuple)
    pub key: FlowKey,

    /// First seen timestamp
    pub first_seen: DateTime<Utc>,

    /// Last seen timestamp (unix timestamp)
    pub last_seen: AtomicU64,

    /// Total packet count
    pub packet_count: AtomicU64,

    /// Total byte count
    pub byte_count: AtomicU64,

    /// TCP flags seen (bitwise OR of all flags)
    pub tcp_flags_seen: AtomicU8,

    /// Dirty flag
    pub dirty: std::sync::atomic::AtomicBool,
}

impl FlowState {
    /// Create a new flow state
    pub fn new(key: FlowKey, now: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            key,
            first_seen: now,
            last_seen: AtomicU64::new(now.timestamp() as u64),
            packet_count: AtomicU64::new(0),
            byte_count: AtomicU64::new(0),
            tcp_flags_seen: AtomicU8::new(0),
            dirty: std::sync::atomic::AtomicBool::new(true),
        }
    }

    /// Update flow state with new packet
    pub fn update(&self, bytes: u64, tcp_flags: Option<u8>, now_ts: u64) {
        self.last_seen.store(now_ts, Ordering::Relaxed);
        self.packet_count.fetch_add(1, Ordering::Relaxed);
        self.byte_count.fetch_add(bytes, Ordering::Relaxed);

        if let Some(flags) = tcp_flags {
            // Bitwise OR to accumulate all seen flags
            self.tcp_flags_seen.fetch_or(flags, Ordering::Relaxed);
        }

        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Check if flow is timed out
    pub fn is_timed_out(&self, timeout_secs: u64) -> bool {
        let now_ts = Utc::now().timestamp() as u64;
        let last_seen = self.last_seen.load(Ordering::Relaxed);
        now_ts.saturating_sub(last_seen) > timeout_secs
    }

    /// Check if this is a TCP connection that has completed (FIN or RST seen)
    pub fn is_tcp_completed(&self) -> bool {
        let flags = self.tcp_flags_seen.load(Ordering::Relaxed);
        // FIN (0x01) or RST (0x04)
        flags & 0x05 != 0
    }

    /// Get duration of the flow in seconds
    pub fn duration_secs(&self) -> u64 {
        let last = self.last_seen.load(Ordering::Relaxed);
        let first = self.first_seen.timestamp() as u64;
        last.saturating_sub(first)
    }

    /// Calculate packets per second
    pub fn packets_per_second(&self) -> f64 {
        let duration = self.duration_secs();
        if duration == 0 {
            return self.packet_count.load(Ordering::Relaxed) as f64;
        }
        self.packet_count.load(Ordering::Relaxed) as f64 / duration as f64
    }

    /// Calculate bytes per second
    pub fn bytes_per_second(&self) -> f64 {
        let duration = self.duration_secs();
        if duration == 0 {
            return self.byte_count.load(Ordering::Relaxed) as f64;
        }
        self.byte_count.load(Ordering::Relaxed) as f64 / duration as f64
    }

    /// Clear dirty flag
    pub fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    /// Check if dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }
}

/// Flow snapshot for persistence
#[derive(Debug, Clone)]
pub struct FlowSnapshot {
    pub id: Uuid,
    pub src_mac: String,
    pub dst_mac: String,
    pub src_ip: Option<Ipv4Addr>,
    pub dst_ip: Option<Ipv4Addr>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub vlan_id: Option<u16>,
    pub ethertype: u16,
    pub ip_protocol: Option<u8>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub packet_count: u64,
    pub byte_count: u64,
    pub tcp_flags_seen: u8,
}

impl FlowState {
    /// Create a snapshot for persistence
    pub fn snapshot(&self, ethertype: u16) -> FlowSnapshot {
        FlowSnapshot {
            id: self.id,
            src_mac: self.key.src_mac.to_string(),
            dst_mac: self.key.dst_mac.to_string(),
            src_ip: self.key.src_ip,
            dst_ip: self.key.dst_ip,
            src_port: self.key.src_port,
            dst_port: self.key.dst_port,
            vlan_id: self.key.vlan_id,
            ethertype,
            ip_protocol: self.key.protocol,
            first_seen: self.first_seen,
            last_seen: DateTime::from_timestamp(self.last_seen.load(Ordering::Relaxed) as i64, 0)
                .unwrap_or(Utc::now()),
            packet_count: self.packet_count.load(Ordering::Relaxed),
            byte_count: self.byte_count.load(Ordering::Relaxed),
            tcp_flags_seen: self.tcp_flags_seen.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_state() {
        let key = FlowKey {
            src_mac: MacAddr::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
            dst_mac: MacAddr::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
            src_ip: Some(Ipv4Addr::new(192, 168, 1, 1)),
            dst_ip: Some(Ipv4Addr::new(192, 168, 1, 2)),
            src_port: Some(12345),
            dst_port: Some(80),
            vlan_id: None,
            protocol: Some(6),
        };

        let flow = FlowState::new(key.clone(), Utc::now());

        // Simulate some packets
        flow.update(100, Some(0x02), Utc::now().timestamp() as u64); // SYN
        flow.update(60, Some(0x12), Utc::now().timestamp() as u64);  // SYN-ACK
        flow.update(52, Some(0x10), Utc::now().timestamp() as u64);  // ACK

        assert_eq!(flow.packet_count.load(Ordering::Relaxed), 3);
        assert_eq!(flow.byte_count.load(Ordering::Relaxed), 212);

        // Check accumulated flags (SYN, ACK)
        let flags = flow.tcp_flags_seen.load(Ordering::Relaxed);
        assert!(flags & 0x02 != 0); // SYN
        assert!(flags & 0x10 != 0); // ACK
    }

    #[test]
    fn test_flow_key_display() {
        let key = FlowKey {
            src_mac: MacAddr::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
            dst_mac: MacAddr::new([0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb]),
            src_ip: Some(Ipv4Addr::new(192, 168, 1, 1)),
            dst_ip: Some(Ipv4Addr::new(10, 0, 0, 1)),
            src_port: Some(54321),
            dst_port: Some(443),
            vlan_id: None,
            protocol: Some(6),
        };

        let display = key.to_display_string();
        assert!(display.contains("192.168.1.1:54321"));
        assert!(display.contains("10.0.0.1:443"));
        assert!(display.contains("TCP"));
    }
}
