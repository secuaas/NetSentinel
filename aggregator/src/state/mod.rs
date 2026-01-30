//! State management for aggregation
//!
//! Uses DashMap for lock-free concurrent access to device and flow state.

pub mod device;
pub mod flow;
pub mod protocol;

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::net::Ipv4Addr;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub use device::{DeviceState, IpState};
pub use flow::{FlowKey, FlowState};
pub use protocol::ProtocolStats;

/// MAC address wrapper for use as a key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddr([u8; 6]);

impl MacAddr {
    pub fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 6 {
            return None;
        }

        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            bytes[i] = u8::from_str_radix(part, 16).ok()?;
        }

        Some(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }

    pub fn to_string(&self) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }

    pub fn oui_prefix(&self) -> String {
        format!("{:02X}:{:02X}:{:02X}", self.0[0], self.0[1], self.0[2])
    }
}

/// Global aggregator state
pub struct AggregatorState {
    /// Device states keyed by MAC address
    pub devices: DashMap<MacAddr, DeviceState>,

    /// Flow states keyed by flow tuple
    pub flows: DashMap<FlowKey, FlowState>,

    /// Protocol statistics
    pub protocols: DashMap<(u16, Option<u8>), ProtocolStats>,

    /// VLAN statistics
    pub vlans: DashMap<u16, VlanStats>,

    // Global counters
    pub total_packets: AtomicU64,
    pub total_bytes: AtomicU64,
    pub total_devices: AtomicU64,
    pub total_flows: AtomicU64,

    /// Start time
    pub start_time: DateTime<Utc>,
}

/// VLAN statistics
pub struct VlanStats {
    pub vlan_id: u16,
    pub outer_vlan_id: Option<u16>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: AtomicU64, // Unix timestamp
    pub packet_count: AtomicU64,
    pub byte_count: AtomicU64,
    pub device_count: AtomicU64,
}

impl AggregatorState {
    /// Create a new aggregator state
    pub fn new() -> Self {
        Self {
            devices: DashMap::new(),
            flows: DashMap::new(),
            protocols: DashMap::new(),
            vlans: DashMap::new(),
            total_packets: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            total_devices: AtomicU64::new(0),
            total_flows: AtomicU64::new(0),
            start_time: Utc::now(),
        }
    }

    /// Process a captured frame
    pub fn process_frame(&self, frame: &CapturedFrame) -> ProcessResult {
        let mut result = ProcessResult::default();

        // Update global counters
        self.total_packets.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(frame.frame_size as u64, Ordering::Relaxed);

        // Parse MAC addresses
        let src_mac = match MacAddr::from_string(&frame.src_mac) {
            Some(m) => m,
            None => return result,
        };
        let dst_mac = match MacAddr::from_string(&frame.dst_mac) {
            Some(m) => m,
            None => return result,
        };

        let now = Utc::now();
        let now_ts = now.timestamp() as u64;

        // Update source device
        let src_is_new = self.update_device(
            src_mac,
            frame.src_ip,
            frame.vlan_id(),
            frame.frame_size as u64,
            true, // is source
            now,
            now_ts,
        );
        if src_is_new {
            result.new_devices.push(src_mac);
        }

        // Update destination device (if not broadcast/multicast)
        if !dst_mac.0[0] & 0x01 == 0x01 {
            let dst_is_new = self.update_device(
                dst_mac,
                frame.dst_ip,
                frame.vlan_id(),
                frame.frame_size as u64,
                false, // is destination
                now,
                now_ts,
            );
            if dst_is_new {
                result.new_devices.push(dst_mac);
            }
        }

        // Update flow
        let flow_key = FlowKey {
            src_mac,
            dst_mac,
            src_ip: frame.src_ip,
            dst_ip: frame.dst_ip,
            src_port: frame.src_port,
            dst_port: frame.dst_port,
            vlan_id: frame.vlan_id(),
            protocol: frame.ip_protocol,
        };

        let flow_is_new = self.update_flow(&flow_key, frame, now, now_ts);
        if flow_is_new {
            result.new_flows.push(flow_key);
        }

        // Update protocol stats
        self.update_protocol(frame.ethertype, frame.ip_protocol, frame.frame_size as u64, now_ts);

        // Update VLAN stats
        if let Some(vlan_id) = frame.vlan_id() {
            self.update_vlan(vlan_id, frame.outer_vlan_id(), frame.frame_size as u64, now, now_ts);
        }

        result
    }

    /// Update or create a device entry
    fn update_device(
        &self,
        mac: MacAddr,
        ip: Option<Ipv4Addr>,
        vlan_id: Option<u16>,
        bytes: u64,
        is_source: bool,
        now: DateTime<Utc>,
        now_ts: u64,
    ) -> bool {
        let mut is_new = false;

        self.devices.entry(mac).or_insert_with(|| {
            is_new = true;
            self.total_devices.fetch_add(1, Ordering::Relaxed);
            DeviceState::new(mac, now)
        }).update(ip, vlan_id, bytes, is_source, now_ts);

        is_new
    }

    /// Update or create a flow entry
    fn update_flow(
        &self,
        key: &FlowKey,
        frame: &CapturedFrame,
        now: DateTime<Utc>,
        now_ts: u64,
    ) -> bool {
        let mut is_new = false;

        self.flows.entry(key.clone()).or_insert_with(|| {
            is_new = true;
            self.total_flows.fetch_add(1, Ordering::Relaxed);
            FlowState::new(key.clone(), now)
        }).update(frame.frame_size as u64, frame.tcp_flags_byte(), now_ts);

        is_new
    }

    /// Update protocol statistics
    fn update_protocol(&self, ethertype: u16, ip_protocol: Option<u8>, bytes: u64, now_ts: u64) {
        self.protocols
            .entry((ethertype, ip_protocol))
            .or_insert_with(|| ProtocolStats::new(ethertype, ip_protocol))
            .update(bytes, now_ts);
    }

    /// Update VLAN statistics
    fn update_vlan(
        &self,
        vlan_id: u16,
        outer_vlan_id: Option<u16>,
        bytes: u64,
        now: DateTime<Utc>,
        now_ts: u64,
    ) {
        self.vlans.entry(vlan_id).or_insert_with(|| VlanStats {
            vlan_id,
            outer_vlan_id,
            first_seen: now,
            last_seen: AtomicU64::new(now_ts),
            packet_count: AtomicU64::new(0),
            byte_count: AtomicU64::new(0),
            device_count: AtomicU64::new(0),
        });

        if let Some(vlan) = self.vlans.get(&vlan_id) {
            vlan.packet_count.fetch_add(1, Ordering::Relaxed);
            vlan.byte_count.fetch_add(bytes, Ordering::Relaxed);
            vlan.last_seen.store(now_ts, Ordering::Relaxed);
        }
    }

    /// Get statistics snapshot
    pub fn stats_snapshot(&self) -> StateStats {
        StateStats {
            total_packets: self.total_packets.load(Ordering::Relaxed),
            total_bytes: self.total_bytes.load(Ordering::Relaxed),
            total_devices: self.devices.len(),
            total_flows: self.flows.len(),
            total_protocols: self.protocols.len(),
            total_vlans: self.vlans.len(),
            uptime_seconds: (Utc::now() - self.start_time).num_seconds() as u64,
        }
    }
}

impl Default for AggregatorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of processing a frame
#[derive(Debug, Default)]
pub struct ProcessResult {
    pub new_devices: Vec<MacAddr>,
    pub new_flows: Vec<FlowKey>,
}

/// State statistics snapshot
#[derive(Debug, Clone)]
pub struct StateStats {
    pub total_packets: u64,
    pub total_bytes: u64,
    pub total_devices: usize,
    pub total_flows: usize,
    pub total_protocols: usize,
    pub total_vlans: usize,
    pub uptime_seconds: u64,
}

/// Captured frame structure (simplified for aggregator)
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CapturedFrame {
    pub timestamp: DateTime<Utc>,
    pub interface: String,
    pub src_mac: String,
    pub dst_mac: String,
    pub ethertype: u16,
    pub vlan: Option<VlanInfo>,
    pub qinq: Option<QinQInfo>,
    pub src_ip: Option<Ipv4Addr>,
    pub dst_ip: Option<Ipv4Addr>,
    pub ip_protocol: Option<u8>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub tcp_flags: Option<TcpFlags>,
    pub frame_size: u32,
    pub payload_size: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct VlanInfo {
    pub id: u16,
    pub priority: u8,
    pub dei: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct QinQInfo {
    pub outer_vlan: VlanInfo,
    pub inner_vlan: VlanInfo,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TcpFlags {
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
}

impl CapturedFrame {
    pub fn vlan_id(&self) -> Option<u16> {
        if let Some(ref qinq) = self.qinq {
            Some(qinq.inner_vlan.id)
        } else {
            self.vlan.as_ref().map(|v| v.id)
        }
    }

    pub fn outer_vlan_id(&self) -> Option<u16> {
        self.qinq.as_ref().map(|q| q.outer_vlan.id)
    }

    pub fn tcp_flags_byte(&self) -> Option<u8> {
        self.tcp_flags.as_ref().map(|f| {
            let mut flags = 0u8;
            if f.fin { flags |= 0x01; }
            if f.syn { flags |= 0x02; }
            if f.rst { flags |= 0x04; }
            if f.psh { flags |= 0x08; }
            if f.ack { flags |= 0x10; }
            if f.urg { flags |= 0x20; }
            flags
        })
    }
}
