//! IPv4 header parsing

use std::net::Ipv4Addr;
use anyhow::{Result, bail};

/// Parsed IPv4 information
#[derive(Debug, Clone)]
pub struct Ipv4Info {
    /// IP version (should be 4)
    pub version: u8,
    /// Header length in bytes
    pub header_length: usize,
    /// Differentiated Services Code Point
    pub dscp: u8,
    /// Explicit Congestion Notification
    pub ecn: u8,
    /// Total length of the IP packet
    pub total_length: u16,
    /// Identification field
    pub identification: u16,
    /// Don't Fragment flag
    pub dont_fragment: bool,
    /// More Fragments flag
    pub more_fragments: bool,
    /// Fragment offset
    pub fragment_offset: u16,
    /// Time To Live
    pub ttl: u8,
    /// Protocol number (6=TCP, 17=UDP, 1=ICMP, etc.)
    pub protocol: u8,
    /// Header checksum
    pub checksum: u16,
    /// Source IP address
    pub src_ip: Ipv4Addr,
    /// Destination IP address
    pub dst_ip: Ipv4Addr,
}

/// IP protocol numbers
pub mod protocol {
    pub const ICMP: u8 = 1;
    pub const IGMP: u8 = 2;
    pub const TCP: u8 = 6;
    pub const UDP: u8 = 17;
    pub const GRE: u8 = 47;
    pub const ESP: u8 = 50;
    pub const AH: u8 = 51;
    pub const ICMPV6: u8 = 58;
    pub const OSPF: u8 = 89;
    pub const SCTP: u8 = 132;
}

/// Get protocol name from number
pub fn protocol_name(protocol: u8) -> &'static str {
    match protocol {
        protocol::ICMP => "ICMP",
        protocol::IGMP => "IGMP",
        protocol::TCP => "TCP",
        protocol::UDP => "UDP",
        protocol::GRE => "GRE",
        protocol::ESP => "ESP",
        protocol::AH => "AH",
        protocol::ICMPV6 => "ICMPv6",
        protocol::OSPF => "OSPF",
        protocol::SCTP => "SCTP",
        _ => "Unknown",
    }
}

/// Parse an IPv4 header
///
/// IPv4 header format:
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |Version|  IHL  |Type of Service|          Total Length         |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |         Identification        |Flags|      Fragment Offset    |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |  Time to Live |    Protocol   |         Header Checksum       |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                       Source Address                          |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Destination Address                        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Options                    |    Padding    |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
pub fn parse_ipv4(data: &[u8]) -> Result<Ipv4Info> {
    if data.len() < 20 {
        bail!("Data too short for IPv4 header: {} bytes (minimum 20)", data.len());
    }

    let version = (data[0] >> 4) & 0x0F;
    if version != 4 {
        bail!("Invalid IP version: {} (expected 4)", version);
    }

    let ihl = (data[0] & 0x0F) as usize;
    let header_length = ihl * 4;

    if header_length < 20 {
        bail!("Invalid IHL: {} (minimum 5)", ihl);
    }

    if data.len() < header_length {
        bail!("Data too short for IPv4 header with options: {} bytes (need {})",
              data.len(), header_length);
    }

    let dscp = (data[1] >> 2) & 0x3F;
    let ecn = data[1] & 0x03;

    let total_length = u16::from_be_bytes([data[2], data[3]]);
    let identification = u16::from_be_bytes([data[4], data[5]]);

    let flags_fragment = u16::from_be_bytes([data[6], data[7]]);
    let dont_fragment = (flags_fragment >> 14) & 0x01 == 1;
    let more_fragments = (flags_fragment >> 13) & 0x01 == 1;
    let fragment_offset = flags_fragment & 0x1FFF;

    let ttl = data[8];
    let protocol = data[9];
    let checksum = u16::from_be_bytes([data[10], data[11]]);

    let src_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
    let dst_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);

    Ok(Ipv4Info {
        version,
        header_length,
        dscp,
        ecn,
        total_length,
        identification,
        dont_fragment,
        more_fragments,
        fragment_offset,
        ttl,
        protocol,
        checksum,
        src_ip,
        dst_ip,
    })
}

/// Check if an IP address is private (RFC 1918)
pub fn is_private(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();

    // 10.0.0.0/8
    if octets[0] == 10 {
        return true;
    }

    // 172.16.0.0/12
    if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
        return true;
    }

    // 192.168.0.0/16
    if octets[0] == 192 && octets[1] == 168 {
        return true;
    }

    false
}

/// Check if an IP address is link-local (169.254.0.0/16)
pub fn is_link_local(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 169 && octets[1] == 254
}

/// Check if an IP address is multicast (224.0.0.0/4)
pub fn is_multicast(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] >= 224 && octets[0] <= 239
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipv4_header() {
        // Simple IPv4 header: version=4, IHL=5, protocol=TCP, src=192.168.1.1, dst=192.168.1.2
        let data = vec![
            0x45, 0x00,             // Version + IHL, DSCP + ECN
            0x00, 0x28,             // Total length (40 bytes)
            0x00, 0x01,             // Identification
            0x40, 0x00,             // Flags (DF=1) + Fragment offset
            0x40, 0x06,             // TTL (64) + Protocol (TCP)
            0x00, 0x00,             // Header checksum
            0xc0, 0xa8, 0x01, 0x01, // Source: 192.168.1.1
            0xc0, 0xa8, 0x01, 0x02, // Destination: 192.168.1.2
        ];

        let info = parse_ipv4(&data).unwrap();

        assert_eq!(info.version, 4);
        assert_eq!(info.header_length, 20);
        assert_eq!(info.ttl, 64);
        assert_eq!(info.protocol, protocol::TCP);
        assert!(info.dont_fragment);
        assert!(!info.more_fragments);
        assert_eq!(info.src_ip.to_string(), "192.168.1.1");
        assert_eq!(info.dst_ip.to_string(), "192.168.1.2");
    }

    #[test]
    fn test_parse_ipv4_with_options() {
        // IPv4 header with options: IHL=6 (24 bytes)
        let mut data = vec![
            0x46, 0x00,             // Version + IHL=6
            0x00, 0x2c,             // Total length (44 bytes)
            0x00, 0x01, 0x40, 0x00, // ID, Flags, Fragment
            0x40, 0x11,             // TTL, Protocol (UDP)
            0x00, 0x00,             // Checksum
            0x0a, 0x00, 0x00, 0x01, // Source: 10.0.0.1
            0x0a, 0x00, 0x00, 0x02, // Destination: 10.0.0.2
            0x00, 0x00, 0x00, 0x00, // Options (4 bytes padding)
        ];

        let info = parse_ipv4(&data).unwrap();

        assert_eq!(info.header_length, 24);
        assert_eq!(info.protocol, protocol::UDP);
    }

    #[test]
    fn test_private_addresses() {
        assert!(is_private(&Ipv4Addr::new(10, 0, 0, 1)));
        assert!(is_private(&Ipv4Addr::new(10, 255, 255, 255)));
        assert!(is_private(&Ipv4Addr::new(172, 16, 0, 1)));
        assert!(is_private(&Ipv4Addr::new(172, 31, 255, 255)));
        assert!(is_private(&Ipv4Addr::new(192, 168, 0, 1)));
        assert!(is_private(&Ipv4Addr::new(192, 168, 255, 255)));

        assert!(!is_private(&Ipv4Addr::new(8, 8, 8, 8)));
        assert!(!is_private(&Ipv4Addr::new(172, 32, 0, 1)));
    }

    #[test]
    fn test_protocol_names() {
        assert_eq!(protocol_name(protocol::TCP), "TCP");
        assert_eq!(protocol_name(protocol::UDP), "UDP");
        assert_eq!(protocol_name(protocol::ICMP), "ICMP");
        assert_eq!(protocol_name(255), "Unknown");
    }

    #[test]
    fn test_invalid_version() {
        let data = vec![
            0x65, 0x00,             // Version=6 (invalid for IPv4)
            0x00, 0x28, 0x00, 0x01, 0x40, 0x00,
            0x40, 0x06, 0x00, 0x00,
            0xc0, 0xa8, 0x01, 0x01,
            0xc0, 0xa8, 0x01, 0x02,
        ];

        assert!(parse_ipv4(&data).is_err());
    }
}
