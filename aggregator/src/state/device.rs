//! Device state management

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::net::Ipv4Addr;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::MacAddr;

/// Device state in memory
pub struct DeviceState {
    /// Unique identifier
    pub id: Uuid,

    /// MAC address
    pub mac: MacAddr,

    /// First seen timestamp
    pub first_seen: DateTime<Utc>,

    /// Last seen timestamp (unix timestamp for atomic updates)
    pub last_seen: AtomicU64,

    /// Total packets sent
    pub packets_sent: AtomicU64,

    /// Total packets received
    pub packets_received: AtomicU64,

    /// Total bytes sent
    pub bytes_sent: AtomicU64,

    /// Total bytes received
    pub bytes_received: AtomicU64,

    /// IP addresses associated with this device
    pub ips: DashMap<Ipv4Addr, IpState>,

    /// VLANs this device has been seen on
    pub vlans: DashMap<u16, ()>,

    /// Whether this device is a gateway
    pub is_gateway: AtomicBool,

    /// Whether this device is flagged for attention
    pub is_flagged: AtomicBool,

    /// Dirty flag (needs to be persisted)
    pub dirty: AtomicBool,
}

/// IP address state for a device
pub struct IpState {
    /// IP address
    pub ip: Ipv4Addr,

    /// VLAN ID (if any)
    pub vlan_id: Option<u16>,

    /// First seen timestamp
    pub first_seen: DateTime<Utc>,

    /// Last seen timestamp
    pub last_seen: AtomicU64,

    /// Packets sent from this IP
    pub packets_sent: AtomicU64,

    /// Packets received to this IP
    pub packets_received: AtomicU64,

    /// Bytes sent from this IP
    pub bytes_sent: AtomicU64,

    /// Bytes received to this IP
    pub bytes_received: AtomicU64,
}

impl DeviceState {
    /// Create a new device state
    pub fn new(mac: MacAddr, now: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            mac,
            first_seen: now,
            last_seen: AtomicU64::new(now.timestamp() as u64),
            packets_sent: AtomicU64::new(0),
            packets_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            ips: DashMap::new(),
            vlans: DashMap::new(),
            is_gateway: AtomicBool::new(false),
            is_flagged: AtomicBool::new(false),
            dirty: AtomicBool::new(true),
        }
    }

    /// Update device state with new packet information
    pub fn update(
        &self,
        ip: Option<Ipv4Addr>,
        vlan_id: Option<u16>,
        bytes: u64,
        is_source: bool,
        now_ts: u64,
    ) {
        // Update timestamps
        self.last_seen.store(now_ts, Ordering::Relaxed);

        // Update counters
        if is_source {
            self.packets_sent.fetch_add(1, Ordering::Relaxed);
            self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
        } else {
            self.packets_received.fetch_add(1, Ordering::Relaxed);
            self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
        }

        // Update IP state
        if let Some(ip_addr) = ip {
            self.update_ip(ip_addr, vlan_id, bytes, is_source, now_ts);
        }

        // Track VLAN
        if let Some(vid) = vlan_id {
            self.vlans.entry(vid).or_insert(());
        }

        // Mark as dirty
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Update IP address state
    fn update_ip(&self, ip: Ipv4Addr, vlan_id: Option<u16>, bytes: u64, is_source: bool, now_ts: u64) {
        self.ips.entry(ip).or_insert_with(|| IpState {
            ip,
            vlan_id,
            first_seen: Utc::now(),
            last_seen: AtomicU64::new(now_ts),
            packets_sent: AtomicU64::new(0),
            packets_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
        });

        if let Some(ip_state) = self.ips.get(&ip) {
            ip_state.last_seen.store(now_ts, Ordering::Relaxed);
            if is_source {
                ip_state.packets_sent.fetch_add(1, Ordering::Relaxed);
                ip_state.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
            } else {
                ip_state.packets_received.fetch_add(1, Ordering::Relaxed);
                ip_state.bytes_received.fetch_add(bytes, Ordering::Relaxed);
            }
        }
    }

    /// Check if device is considered inactive
    pub fn is_inactive(&self, timeout_secs: u64) -> bool {
        let now_ts = Utc::now().timestamp() as u64;
        let last_seen = self.last_seen.load(Ordering::Relaxed);
        now_ts.saturating_sub(last_seen) > timeout_secs
    }

    /// Get total packet count
    pub fn total_packets(&self) -> u64 {
        self.packets_sent.load(Ordering::Relaxed) +
        self.packets_received.load(Ordering::Relaxed)
    }

    /// Get total byte count
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed) +
        self.bytes_received.load(Ordering::Relaxed)
    }

    /// Get list of IP addresses
    pub fn ip_list(&self) -> Vec<Ipv4Addr> {
        self.ips.iter().map(|entry| *entry.key()).collect()
    }

    /// Get list of VLANs
    pub fn vlan_list(&self) -> Vec<u16> {
        self.vlans.iter().map(|entry| *entry.key()).collect()
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

/// Device snapshot for persistence
#[derive(Debug, Clone)]
pub struct DeviceSnapshot {
    pub id: Uuid,
    pub mac_address: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub is_gateway: bool,
    pub is_flagged: bool,
    pub ip_addresses: Vec<IpSnapshot>,
    pub vlans: Vec<u16>,
}

/// IP address snapshot
#[derive(Debug, Clone)]
pub struct IpSnapshot {
    pub ip_address: Ipv4Addr,
    pub vlan_id: Option<u16>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl DeviceState {
    /// Create a snapshot for persistence
    pub fn snapshot(&self) -> DeviceSnapshot {
        let ip_addresses: Vec<IpSnapshot> = self.ips.iter().map(|entry| {
            let ip_state = entry.value();
            IpSnapshot {
                ip_address: ip_state.ip,
                vlan_id: ip_state.vlan_id,
                first_seen: ip_state.first_seen,
                last_seen: DateTime::from_timestamp(ip_state.last_seen.load(Ordering::Relaxed) as i64, 0)
                    .unwrap_or(Utc::now()),
                packets_sent: ip_state.packets_sent.load(Ordering::Relaxed),
                packets_received: ip_state.packets_received.load(Ordering::Relaxed),
                bytes_sent: ip_state.bytes_sent.load(Ordering::Relaxed),
                bytes_received: ip_state.bytes_received.load(Ordering::Relaxed),
            }
        }).collect();

        DeviceSnapshot {
            id: self.id,
            mac_address: self.mac.to_string(),
            first_seen: self.first_seen,
            last_seen: DateTime::from_timestamp(self.last_seen.load(Ordering::Relaxed) as i64, 0)
                .unwrap_or(Utc::now()),
            packets_sent: self.packets_sent.load(Ordering::Relaxed),
            packets_received: self.packets_received.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            is_gateway: self.is_gateway.load(Ordering::Relaxed),
            is_flagged: self.is_flagged.load(Ordering::Relaxed),
            ip_addresses,
            vlans: self.vlan_list(),
        }
    }
}
