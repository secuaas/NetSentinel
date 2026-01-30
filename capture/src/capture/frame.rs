//! Frame data structures for captured network packets

use std::fmt;
use std::net::Ipv4Addr;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize, Serializer, Deserializer};

/// MAC address (6 bytes)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddr([u8; 6]);

impl MacAddr {
    /// Create a new MAC address from bytes
    pub fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Create from a slice (must be exactly 6 bytes)
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() == 6 {
            let mut bytes = [0u8; 6];
            bytes.copy_from_slice(slice);
            Some(Self(bytes))
        } else {
            None
        }
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }

    /// Check if this is a broadcast address
    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
    }

    /// Check if this is a multicast address (bit 0 of first byte is 1)
    pub fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 == 0x01
    }

    /// Check if this is a locally administered address
    pub fn is_local(&self) -> bool {
        self.0[0] & 0x02 == 0x02
    }

    /// Get the OUI prefix as a string (XX:XX:XX)
    pub fn oui_prefix(&self) -> String {
        format!("{:02X}:{:02X}:{:02X}", self.0[0], self.0[1], self.0[2])
    }

    /// Get the OUI prefix as bytes
    pub fn oui_bytes(&self) -> [u8; 3] {
        [self.0[0], self.0[1], self.0[2]]
    }
}

impl fmt::Debug for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MacAddr({})", self)
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl Serialize for MacAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MacAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 6 {
            return Err(serde::de::Error::custom("Invalid MAC address format"));
        }

        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            bytes[i] = u8::from_str_radix(part, 16)
                .map_err(|_| serde::de::Error::custom("Invalid MAC address byte"))?;
        }

        Ok(MacAddr(bytes))
    }
}

/// VLAN information (802.1Q)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlanInfo {
    /// VLAN ID (12 bits, 0-4095)
    pub id: u16,
    /// Priority Code Point (3 bits, 0-7)
    pub priority: u8,
    /// Drop Eligible Indicator
    pub dei: bool,
}

impl VlanInfo {
    /// Parse VLAN tag from 2 bytes (TCI field)
    pub fn from_tci(tci: u16) -> Self {
        Self {
            id: tci & 0x0FFF,
            priority: ((tci >> 13) & 0x07) as u8,
            dei: (tci >> 12) & 0x01 == 1,
        }
    }
}

/// QinQ (802.1ad) double VLAN tagging information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QinQInfo {
    /// Outer VLAN (S-VLAN / Service VLAN)
    pub outer_vlan: VlanInfo,
    /// Inner VLAN (C-VLAN / Customer VLAN)
    pub inner_vlan: VlanInfo,
}

/// TCP flags
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TcpFlags {
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
    pub ece: bool,
    pub cwr: bool,
}

impl TcpFlags {
    /// Parse TCP flags from the flags byte
    pub fn from_byte(flags: u8) -> Self {
        Self {
            fin: flags & 0x01 != 0,
            syn: flags & 0x02 != 0,
            rst: flags & 0x04 != 0,
            psh: flags & 0x08 != 0,
            ack: flags & 0x10 != 0,
            urg: flags & 0x20 != 0,
            ece: flags & 0x40 != 0,
            cwr: flags & 0x80 != 0,
        }
    }

    /// Convert flags back to a byte
    pub fn to_byte(&self) -> u8 {
        let mut flags = 0u8;
        if self.fin { flags |= 0x01; }
        if self.syn { flags |= 0x02; }
        if self.rst { flags |= 0x04; }
        if self.psh { flags |= 0x08; }
        if self.ack { flags |= 0x10; }
        if self.urg { flags |= 0x20; }
        if self.ece { flags |= 0x40; }
        if self.cwr { flags |= 0x80; }
        flags
    }

    /// Check if this is a SYN-only packet (connection initiation)
    pub fn is_syn_only(&self) -> bool {
        self.syn && !self.ack
    }

    /// Check if this is a SYN-ACK packet
    pub fn is_syn_ack(&self) -> bool {
        self.syn && self.ack
    }
}

impl fmt::Display for TcpFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = String::new();
        if self.syn { flags.push('S'); }
        if self.ack { flags.push('A'); }
        if self.fin { flags.push('F'); }
        if self.rst { flags.push('R'); }
        if self.psh { flags.push('P'); }
        if self.urg { flags.push('U'); }
        if self.ece { flags.push('E'); }
        if self.cwr { flags.push('C'); }
        if flags.is_empty() {
            flags.push_str("none");
        }
        write!(f, "[{}]", flags)
    }
}

/// Captured frame with all parsed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedFrame {
    /// Capture timestamp
    pub timestamp: DateTime<Utc>,

    /// Interface name where the frame was captured
    pub interface: String,

    // Layer 2 - Ethernet
    /// Source MAC address
    pub src_mac: MacAddr,

    /// Destination MAC address
    pub dst_mac: MacAddr,

    /// EtherType (0x0800 = IPv4, 0x0806 = ARP, 0x86DD = IPv6, etc.)
    pub ethertype: u16,

    /// VLAN information (if 802.1Q tagged)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan: Option<VlanInfo>,

    /// QinQ information (if 802.1ad double-tagged)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qinq: Option<QinQInfo>,

    // Layer 3 - IP
    /// Source IP address (IPv4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_ip: Option<Ipv4Addr>,

    /// Destination IP address (IPv4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_ip: Option<Ipv4Addr>,

    /// IP protocol number (6 = TCP, 17 = UDP, 1 = ICMP, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_protocol: Option<u8>,

    /// Time To Live
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u8>,

    // Layer 4 - Transport
    /// Source port (TCP/UDP)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_port: Option<u16>,

    /// Destination port (TCP/UDP)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_port: Option<u16>,

    /// TCP flags (if TCP)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_flags: Option<TcpFlags>,

    // Metadata
    /// Total frame size in bytes
    pub frame_size: u32,

    /// Payload size (after headers)
    pub payload_size: u32,
}

impl CapturedFrame {
    /// Create a new empty frame with basic info
    pub fn new(interface: &str, src_mac: MacAddr, dst_mac: MacAddr, ethertype: u16, frame_size: u32) -> Self {
        Self {
            timestamp: Utc::now(),
            interface: interface.to_string(),
            src_mac,
            dst_mac,
            ethertype,
            vlan: None,
            qinq: None,
            src_ip: None,
            dst_ip: None,
            ip_protocol: None,
            ttl: None,
            src_port: None,
            dst_port: None,
            tcp_flags: None,
            frame_size,
            payload_size: 0,
        }
    }

    /// Check if this frame is IPv4
    pub fn is_ipv4(&self) -> bool {
        self.ethertype == 0x0800
    }

    /// Check if this frame is ARP
    pub fn is_arp(&self) -> bool {
        self.ethertype == 0x0806
    }

    /// Check if this frame is IPv6
    pub fn is_ipv6(&self) -> bool {
        self.ethertype == 0x86DD
    }

    /// Check if this frame is TCP
    pub fn is_tcp(&self) -> bool {
        self.ip_protocol == Some(6)
    }

    /// Check if this frame is UDP
    pub fn is_udp(&self) -> bool {
        self.ip_protocol == Some(17)
    }

    /// Check if this frame is ICMP
    pub fn is_icmp(&self) -> bool {
        self.ip_protocol == Some(1)
    }

    /// Get the VLAN ID (inner VLAN if QinQ)
    pub fn vlan_id(&self) -> Option<u16> {
        if let Some(ref qinq) = self.qinq {
            Some(qinq.inner_vlan.id)
        } else {
            self.vlan.as_ref().map(|v| v.id)
        }
    }

    /// Get the outer VLAN ID (for QinQ)
    pub fn outer_vlan_id(&self) -> Option<u16> {
        self.qinq.as_ref().map(|q| q.outer_vlan.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mac_addr() {
        let mac = MacAddr::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert_eq!(mac.to_string(), "00:11:22:33:44:55");
        assert_eq!(mac.oui_prefix(), "00:11:22");
        assert!(!mac.is_broadcast());
        assert!(!mac.is_multicast());

        let broadcast = MacAddr::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
        assert!(broadcast.is_broadcast());
        assert!(broadcast.is_multicast());

        let multicast = MacAddr::new([0x01, 0x00, 0x5e, 0x00, 0x00, 0x01]);
        assert!(!multicast.is_broadcast());
        assert!(multicast.is_multicast());
    }

    #[test]
    fn test_vlan_info() {
        // TCI: Priority=5, DEI=0, VID=100
        // Binary: 101 0 000001100100 = 0xA064
        let tci: u16 = 0xA064;
        let vlan = VlanInfo::from_tci(tci);
        assert_eq!(vlan.id, 100);
        assert_eq!(vlan.priority, 5);
        assert!(!vlan.dei);
    }

    #[test]
    fn test_tcp_flags() {
        let syn = TcpFlags::from_byte(0x02);
        assert!(syn.syn);
        assert!(!syn.ack);
        assert!(syn.is_syn_only());

        let syn_ack = TcpFlags::from_byte(0x12);
        assert!(syn_ack.syn);
        assert!(syn_ack.ack);
        assert!(syn_ack.is_syn_ack());

        assert_eq!(syn.to_byte(), 0x02);
        assert_eq!(syn_ack.to_byte(), 0x12);
    }

    #[test]
    fn test_mac_addr_serialization() {
        let mac = MacAddr::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        let json = serde_json::to_string(&mac).unwrap();
        assert_eq!(json, "\"00:11:22:33:44:55\"");

        let parsed: MacAddr = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, mac);
    }
}
