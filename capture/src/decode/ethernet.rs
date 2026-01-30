//! Ethernet frame parsing

use anyhow::{Result, bail};
use crate::capture::frame::{CapturedFrame, MacAddr, VlanInfo, QinQInfo};

// EtherType constants
pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16 = 0x0806;
pub const ETHERTYPE_VLAN: u16 = 0x8100;      // 802.1Q
pub const ETHERTYPE_QINQ: u16 = 0x88A8;      // 802.1ad (QinQ outer)
pub const ETHERTYPE_QINQ_ALT: u16 = 0x9100;  // Alternative QinQ tag
pub const ETHERTYPE_IPV6: u16 = 0x86DD;
pub const ETHERTYPE_MPLS: u16 = 0x8847;
pub const ETHERTYPE_LLDP: u16 = 0x88CC;

/// Minimum Ethernet frame size (without preamble/FCS)
pub const MIN_FRAME_SIZE: usize = 14;

/// Parse an Ethernet frame header
pub fn parse_ethernet(data: &[u8]) -> Result<(MacAddr, MacAddr, u16, usize)> {
    if data.len() < MIN_FRAME_SIZE {
        bail!("Frame too short: {} bytes (minimum {})", data.len(), MIN_FRAME_SIZE);
    }

    let dst_mac = MacAddr::from_slice(&data[0..6])
        .ok_or_else(|| anyhow::anyhow!("Failed to parse destination MAC"))?;

    let src_mac = MacAddr::from_slice(&data[6..12])
        .ok_or_else(|| anyhow::anyhow!("Failed to parse source MAC"))?;

    let ethertype = u16::from_be_bytes([data[12], data[13]]);

    Ok((dst_mac, src_mac, ethertype, 14))
}

/// Parse a complete frame from raw bytes
pub fn parse_frame(interface: &str, data: &[u8]) -> Result<CapturedFrame> {
    let frame_size = data.len() as u32;

    // Parse Ethernet header
    let (dst_mac, src_mac, mut ethertype, mut offset) = parse_ethernet(data)?;

    // Create frame with basic info
    let mut frame = CapturedFrame::new(interface, src_mac, dst_mac, ethertype, frame_size);

    // Handle VLAN tags (802.1Q and 802.1ad QinQ)
    match ethertype {
        ETHERTYPE_QINQ | ETHERTYPE_QINQ_ALT => {
            // QinQ: Parse outer VLAN
            if data.len() < offset + 4 {
                bail!("Frame too short for QinQ outer tag");
            }

            let outer_tci = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let outer_vlan = VlanInfo::from_tci(outer_tci);
            let inner_ethertype = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
            offset += 4;

            // Check for inner VLAN (802.1Q)
            if inner_ethertype == ETHERTYPE_VLAN {
                if data.len() < offset + 4 {
                    bail!("Frame too short for QinQ inner tag");
                }

                let inner_tci = u16::from_be_bytes([data[offset], data[offset + 1]]);
                let inner_vlan = VlanInfo::from_tci(inner_tci);
                ethertype = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
                offset += 4;

                frame.qinq = Some(QinQInfo {
                    outer_vlan,
                    inner_vlan,
                });
            } else {
                // Single outer tag (unusual but possible)
                frame.vlan = Some(outer_vlan);
                ethertype = inner_ethertype;
            }
        }
        ETHERTYPE_VLAN => {
            // Single VLAN tag (802.1Q)
            if data.len() < offset + 4 {
                bail!("Frame too short for VLAN tag");
            }

            let tci = u16::from_be_bytes([data[offset], data[offset + 1]]);
            frame.vlan = Some(VlanInfo::from_tci(tci));
            ethertype = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
            offset += 4;
        }
        _ => {}
    }

    // Update ethertype after VLAN processing
    frame.ethertype = ethertype;

    // Parse Layer 3 based on ethertype
    if ethertype == ETHERTYPE_IPV4 && data.len() > offset {
        if let Ok(ip_info) = super::ipv4::parse_ipv4(&data[offset..]) {
            frame.src_ip = Some(ip_info.src_ip);
            frame.dst_ip = Some(ip_info.dst_ip);
            frame.ip_protocol = Some(ip_info.protocol);
            frame.ttl = Some(ip_info.ttl);

            // Parse transport layer
            let transport_offset = offset + ip_info.header_length;
            if data.len() > transport_offset {
                if let Ok(transport_info) = super::transport::parse_transport(
                    ip_info.protocol,
                    &data[transport_offset..],
                ) {
                    frame.src_port = transport_info.src_port;
                    frame.dst_port = transport_info.dst_port;
                    frame.tcp_flags = transport_info.tcp_flags;
                    frame.payload_size = transport_info.payload_size;
                }
            }
        }
    }

    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ethernet_header() {
        // Ethernet frame: dst=ff:ff:ff:ff:ff:ff, src=00:11:22:33:44:55, ethertype=0x0800 (IPv4)
        let data = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst MAC (broadcast)
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src MAC
            0x08, 0x00,                         // EtherType (IPv4)
        ];

        let (dst, src, ethertype, offset) = parse_ethernet(&data).unwrap();

        assert!(dst.is_broadcast());
        assert_eq!(src.to_string(), "00:11:22:33:44:55");
        assert_eq!(ethertype, ETHERTYPE_IPV4);
        assert_eq!(offset, 14);
    }

    #[test]
    fn test_parse_vlan_frame() {
        // Ethernet frame with VLAN tag
        let data = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst MAC
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src MAC
            0x81, 0x00,                         // EtherType (802.1Q)
            0x00, 0x64,                         // TCI: VLAN ID 100
            0x08, 0x00,                         // Inner EtherType (IPv4)
        ];

        let frame = parse_frame("eth0", &data).unwrap();

        assert!(frame.vlan.is_some());
        assert_eq!(frame.vlan.as_ref().unwrap().id, 100);
        assert_eq!(frame.ethertype, ETHERTYPE_IPV4);
    }

    #[test]
    fn test_parse_qinq_frame() {
        // Ethernet frame with QinQ (802.1ad)
        let data = vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst MAC
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src MAC
            0x88, 0xa8,                         // EtherType (802.1ad)
            0x00, 0xc8,                         // Outer VLAN ID 200
            0x81, 0x00,                         // Inner 802.1Q
            0x00, 0x64,                         // Inner VLAN ID 100
            0x08, 0x00,                         // Final EtherType (IPv4)
        ];

        let frame = parse_frame("eth0", &data).unwrap();

        assert!(frame.qinq.is_some());
        let qinq = frame.qinq.as_ref().unwrap();
        assert_eq!(qinq.outer_vlan.id, 200);
        assert_eq!(qinq.inner_vlan.id, 100);
        assert_eq!(frame.ethertype, ETHERTYPE_IPV4);
    }

    #[test]
    fn test_frame_too_short() {
        let data = vec![0xff, 0xff, 0xff]; // Only 3 bytes
        assert!(parse_ethernet(&data).is_err());
    }
}
