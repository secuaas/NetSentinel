//! Protocol statistics

use std::sync::atomic::{AtomicU64, Ordering};
use chrono::{DateTime, Utc};

/// Protocol statistics
pub struct ProtocolStats {
    /// EtherType (e.g., 0x0800 for IPv4)
    pub ethertype: u16,

    /// IP protocol number (if applicable)
    pub ip_protocol: Option<u8>,

    /// First seen timestamp
    pub first_seen: DateTime<Utc>,

    /// Last seen timestamp (unix timestamp)
    pub last_seen: AtomicU64,

    /// Total packet count
    pub packet_count: AtomicU64,

    /// Total byte count
    pub byte_count: AtomicU64,
}

impl ProtocolStats {
    /// Create new protocol stats
    pub fn new(ethertype: u16, ip_protocol: Option<u8>) -> Self {
        let now = Utc::now();
        Self {
            ethertype,
            ip_protocol,
            first_seen: now,
            last_seen: AtomicU64::new(now.timestamp() as u64),
            packet_count: AtomicU64::new(0),
            byte_count: AtomicU64::new(0),
        }
    }

    /// Update statistics
    pub fn update(&self, bytes: u64, now_ts: u64) {
        self.packet_count.fetch_add(1, Ordering::Relaxed);
        self.byte_count.fetch_add(bytes, Ordering::Relaxed);
        self.last_seen.store(now_ts, Ordering::Relaxed);
    }

    /// Get protocol name
    pub fn name(&self) -> &'static str {
        match self.ethertype {
            0x0800 => match self.ip_protocol {
                Some(1) => "ICMP",
                Some(2) => "IGMP",
                Some(6) => "TCP",
                Some(17) => "UDP",
                Some(47) => "GRE",
                Some(50) => "ESP",
                Some(51) => "AH",
                Some(89) => "OSPF",
                Some(132) => "SCTP",
                Some(_) => "IPv4/Other",
                None => "IPv4",
            },
            0x0806 => "ARP",
            0x8100 => "VLAN",
            0x86DD => "IPv6",
            0x8847 => "MPLS",
            0x88A8 => "QinQ",
            0x88CC => "LLDP",
            0x8906 => "FCoE",
            _ => "Unknown",
        }
    }
}

/// Protocol statistics snapshot
#[derive(Debug, Clone)]
pub struct ProtocolSnapshot {
    pub ethertype: u16,
    pub ip_protocol: Option<u8>,
    pub protocol_name: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub packet_count: u64,
    pub byte_count: u64,
}

impl ProtocolStats {
    /// Create a snapshot for reporting
    pub fn snapshot(&self) -> ProtocolSnapshot {
        ProtocolSnapshot {
            ethertype: self.ethertype,
            ip_protocol: self.ip_protocol,
            protocol_name: self.name().to_string(),
            first_seen: self.first_seen,
            last_seen: DateTime::from_timestamp(self.last_seen.load(Ordering::Relaxed) as i64, 0)
                .unwrap_or(Utc::now()),
            packet_count: self.packet_count.load(Ordering::Relaxed),
            byte_count: self.byte_count.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_names() {
        let tcp = ProtocolStats::new(0x0800, Some(6));
        assert_eq!(tcp.name(), "TCP");

        let udp = ProtocolStats::new(0x0800, Some(17));
        assert_eq!(udp.name(), "UDP");

        let arp = ProtocolStats::new(0x0806, None);
        assert_eq!(arp.name(), "ARP");

        let ipv6 = ProtocolStats::new(0x86DD, None);
        assert_eq!(ipv6.name(), "IPv6");
    }

    #[test]
    fn test_protocol_stats_update() {
        let stats = ProtocolStats::new(0x0800, Some(6));

        stats.update(100, Utc::now().timestamp() as u64);
        stats.update(200, Utc::now().timestamp() as u64);

        assert_eq!(stats.packet_count.load(Ordering::Relaxed), 2);
        assert_eq!(stats.byte_count.load(Ordering::Relaxed), 300);
    }
}
