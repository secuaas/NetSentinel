//! VLAN tag parsing (802.1Q and 802.1ad)

use anyhow::{Result, bail};
use crate::capture::frame::{VlanInfo, QinQInfo};

/// Parse a single VLAN tag (802.1Q)
///
/// The VLAN tag is 4 bytes:
/// - 2 bytes: TPID (Tag Protocol Identifier) = 0x8100
/// - 2 bytes: TCI (Tag Control Information)
///   - 3 bits: PCP (Priority Code Point)
///   - 1 bit: DEI (Drop Eligible Indicator)
///   - 12 bits: VID (VLAN Identifier)
pub fn parse_vlan(data: &[u8]) -> Result<(VlanInfo, u16, usize)> {
    if data.len() < 4 {
        bail!("Data too short for VLAN tag: {} bytes", data.len());
    }

    let tci = u16::from_be_bytes([data[0], data[1]]);
    let ethertype = u16::from_be_bytes([data[2], data[3]]);

    Ok((VlanInfo::from_tci(tci), ethertype, 4))
}

/// Parse QinQ double VLAN tags (802.1ad)
///
/// QinQ encapsulation:
/// - Outer tag: S-VLAN (Service VLAN) with TPID 0x88A8
/// - Inner tag: C-VLAN (Customer VLAN) with TPID 0x8100
pub fn parse_qinq(data: &[u8]) -> Result<(QinQInfo, u16, usize)> {
    if data.len() < 8 {
        bail!("Data too short for QinQ tags: {} bytes", data.len());
    }

    // Outer VLAN (S-VLAN)
    let outer_tci = u16::from_be_bytes([data[0], data[1]]);
    let inner_tpid = u16::from_be_bytes([data[2], data[3]]);

    // Verify inner TPID is 802.1Q
    if inner_tpid != 0x8100 {
        bail!("Invalid inner TPID for QinQ: 0x{:04x}", inner_tpid);
    }

    // Inner VLAN (C-VLAN)
    let inner_tci = u16::from_be_bytes([data[4], data[5]]);
    let ethertype = u16::from_be_bytes([data[6], data[7]]);

    let qinq = QinQInfo {
        outer_vlan: VlanInfo::from_tci(outer_tci),
        inner_vlan: VlanInfo::from_tci(inner_tci),
    };

    Ok((qinq, ethertype, 8))
}

/// Get VLAN priority name
pub fn priority_name(priority: u8) -> &'static str {
    match priority {
        0 => "Best Effort (BE)",
        1 => "Background (BK)",
        2 => "Excellent Effort (EE)",
        3 => "Critical Applications (CA)",
        4 => "Video (VI)",
        5 => "Voice (VO)",
        6 => "Internetwork Control (IC)",
        7 => "Network Control (NC)",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vlan() {
        // TCI: PCP=3, DEI=0, VID=100
        // Binary: 011 0 000001100100 = 0x6064
        let data = vec![0x60, 0x64, 0x08, 0x00];

        let (vlan, ethertype, offset) = parse_vlan(&data).unwrap();

        assert_eq!(vlan.id, 100);
        assert_eq!(vlan.priority, 3);
        assert!(!vlan.dei);
        assert_eq!(ethertype, 0x0800);
        assert_eq!(offset, 4);
    }

    #[test]
    fn test_parse_qinq() {
        // Outer: VID=200, Inner: VID=100
        let data = vec![
            0x00, 0xc8, // Outer TCI (VID=200)
            0x81, 0x00, // Inner TPID (802.1Q)
            0x00, 0x64, // Inner TCI (VID=100)
            0x08, 0x00, // EtherType (IPv4)
        ];

        let (qinq, ethertype, offset) = parse_qinq(&data).unwrap();

        assert_eq!(qinq.outer_vlan.id, 200);
        assert_eq!(qinq.inner_vlan.id, 100);
        assert_eq!(ethertype, 0x0800);
        assert_eq!(offset, 8);
    }

    #[test]
    fn test_vlan_with_priority() {
        // TCI: PCP=5, DEI=1, VID=42
        // Binary: 101 1 000000101010 = 0xB02A
        let data = vec![0xB0, 0x2A, 0x08, 0x00];

        let (vlan, _, _) = parse_vlan(&data).unwrap();

        assert_eq!(vlan.id, 42);
        assert_eq!(vlan.priority, 5);
        assert!(vlan.dei);
    }

    #[test]
    fn test_priority_names() {
        assert_eq!(priority_name(0), "Best Effort (BE)");
        assert_eq!(priority_name(5), "Voice (VO)");
        assert_eq!(priority_name(7), "Network Control (NC)");
    }
}
