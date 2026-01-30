//! Transport layer (TCP/UDP) parsing

use anyhow::{Result, bail};
use crate::capture::frame::TcpFlags;
use super::ipv4::protocol;

/// Parsed transport layer information
#[derive(Debug, Clone)]
pub struct TransportInfo {
    /// Source port (TCP/UDP)
    pub src_port: Option<u16>,
    /// Destination port (TCP/UDP)
    pub dst_port: Option<u16>,
    /// TCP flags (if TCP)
    pub tcp_flags: Option<TcpFlags>,
    /// TCP sequence number (if TCP)
    pub tcp_seq: Option<u32>,
    /// TCP acknowledgment number (if TCP)
    pub tcp_ack: Option<u32>,
    /// TCP window size (if TCP)
    pub tcp_window: Option<u16>,
    /// Payload size after transport header
    pub payload_size: u32,
}

/// Well-known port numbers
pub mod ports {
    pub const FTP_DATA: u16 = 20;
    pub const FTP: u16 = 21;
    pub const SSH: u16 = 22;
    pub const TELNET: u16 = 23;
    pub const SMTP: u16 = 25;
    pub const DNS: u16 = 53;
    pub const DHCP_SERVER: u16 = 67;
    pub const DHCP_CLIENT: u16 = 68;
    pub const HTTP: u16 = 80;
    pub const KERBEROS: u16 = 88;
    pub const POP3: u16 = 110;
    pub const NTP: u16 = 123;
    pub const NETBIOS_NS: u16 = 137;
    pub const NETBIOS_DGM: u16 = 138;
    pub const NETBIOS_SSN: u16 = 139;
    pub const IMAP: u16 = 143;
    pub const SNMP: u16 = 161;
    pub const SNMP_TRAP: u16 = 162;
    pub const LDAP: u16 = 389;
    pub const HTTPS: u16 = 443;
    pub const SMB: u16 = 445;
    pub const LDAPS: u16 = 636;
    pub const IMAPS: u16 = 993;
    pub const MYSQL: u16 = 3306;
    pub const RDP: u16 = 3389;
    pub const POSTGRESQL: u16 = 5432;
    pub const REDIS: u16 = 6379;
    pub const HTTP_ALT: u16 = 8080;
    pub const HTTPS_ALT: u16 = 8443;
}

/// Get service name from port number
pub fn service_name(port: u16) -> Option<&'static str> {
    match port {
        ports::FTP_DATA => Some("ftp-data"),
        ports::FTP => Some("ftp"),
        ports::SSH => Some("ssh"),
        ports::TELNET => Some("telnet"),
        ports::SMTP => Some("smtp"),
        ports::DNS => Some("dns"),
        ports::DHCP_SERVER | ports::DHCP_CLIENT => Some("dhcp"),
        ports::HTTP | ports::HTTP_ALT => Some("http"),
        ports::KERBEROS => Some("kerberos"),
        ports::POP3 => Some("pop3"),
        ports::NTP => Some("ntp"),
        ports::NETBIOS_NS | ports::NETBIOS_DGM | ports::NETBIOS_SSN => Some("netbios"),
        ports::IMAP => Some("imap"),
        ports::SNMP | ports::SNMP_TRAP => Some("snmp"),
        ports::LDAP => Some("ldap"),
        ports::HTTPS | ports::HTTPS_ALT => Some("https"),
        ports::SMB => Some("smb"),
        ports::LDAPS => Some("ldaps"),
        ports::IMAPS => Some("imaps"),
        ports::MYSQL => Some("mysql"),
        ports::RDP => Some("rdp"),
        ports::POSTGRESQL => Some("postgresql"),
        ports::REDIS => Some("redis"),
        _ => None,
    }
}

/// Parse transport layer header
pub fn parse_transport(ip_protocol: u8, data: &[u8]) -> Result<TransportInfo> {
    match ip_protocol {
        protocol::TCP => parse_tcp(data),
        protocol::UDP => parse_udp(data),
        _ => Ok(TransportInfo {
            src_port: None,
            dst_port: None,
            tcp_flags: None,
            tcp_seq: None,
            tcp_ack: None,
            tcp_window: None,
            payload_size: data.len() as u32,
        }),
    }
}

/// Parse TCP header
///
/// TCP header format:
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |          Source Port          |       Destination Port        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                        Sequence Number                        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Acknowledgment Number                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |  Data |           |U|A|P|R|S|F|                               |
/// | Offset| Reserved  |R|C|S|S|Y|I|            Window             |
/// |       |           |G|K|H|T|N|N|                               |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |           Checksum            |         Urgent Pointer        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                    Options                    |    Padding    |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
fn parse_tcp(data: &[u8]) -> Result<TransportInfo> {
    if data.len() < 20 {
        bail!("Data too short for TCP header: {} bytes (minimum 20)", data.len());
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let seq = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let ack = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

    let data_offset = ((data[12] >> 4) & 0x0F) as usize * 4;
    if data_offset < 20 {
        bail!("Invalid TCP data offset: {} (minimum 20)", data_offset);
    }

    let flags = TcpFlags::from_byte(data[13]);
    let window = u16::from_be_bytes([data[14], data[15]]);

    let payload_size = if data.len() > data_offset {
        (data.len() - data_offset) as u32
    } else {
        0
    };

    Ok(TransportInfo {
        src_port: Some(src_port),
        dst_port: Some(dst_port),
        tcp_flags: Some(flags),
        tcp_seq: Some(seq),
        tcp_ack: Some(ack),
        tcp_window: Some(window),
        payload_size,
    })
}

/// Parse UDP header
///
/// UDP header format:
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |          Source Port          |       Destination Port        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |            Length             |           Checksum            |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
fn parse_udp(data: &[u8]) -> Result<TransportInfo> {
    if data.len() < 8 {
        bail!("Data too short for UDP header: {} bytes (minimum 8)", data.len());
    }

    let src_port = u16::from_be_bytes([data[0], data[1]]);
    let dst_port = u16::from_be_bytes([data[2], data[3]]);
    let length = u16::from_be_bytes([data[4], data[5]]);

    // UDP length includes header (8 bytes)
    let payload_size = if length > 8 { length - 8 } else { 0 };

    Ok(TransportInfo {
        src_port: Some(src_port),
        dst_port: Some(dst_port),
        tcp_flags: None,
        tcp_seq: None,
        tcp_ack: None,
        tcp_window: None,
        payload_size: payload_size as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tcp_header() {
        // TCP header: src=443, dst=54321, SYN flag
        let data = vec![
            0x01, 0xbb,             // Source port: 443
            0xd4, 0x31,             // Destination port: 54321
            0x00, 0x00, 0x00, 0x01, // Sequence number: 1
            0x00, 0x00, 0x00, 0x00, // Ack number: 0
            0x50, 0x02,             // Data offset (5), Flags (SYN)
            0xff, 0xff,             // Window: 65535
            0x00, 0x00,             // Checksum
            0x00, 0x00,             // Urgent pointer
        ];

        let info = parse_tcp(&data).unwrap();

        assert_eq!(info.src_port, Some(443));
        assert_eq!(info.dst_port, Some(54321));
        assert!(info.tcp_flags.as_ref().unwrap().syn);
        assert!(!info.tcp_flags.as_ref().unwrap().ack);
        assert_eq!(info.tcp_seq, Some(1));
        assert_eq!(info.tcp_window, Some(65535));
    }

    #[test]
    fn test_parse_udp_header() {
        // UDP header: src=53, dst=12345, length=100
        let data = vec![
            0x00, 0x35,             // Source port: 53 (DNS)
            0x30, 0x39,             // Destination port: 12345
            0x00, 0x64,             // Length: 100
            0x00, 0x00,             // Checksum
        ];

        let info = parse_udp(&data).unwrap();

        assert_eq!(info.src_port, Some(53));
        assert_eq!(info.dst_port, Some(12345));
        assert!(info.tcp_flags.is_none());
        assert_eq!(info.payload_size, 92); // 100 - 8 header
    }

    #[test]
    fn test_service_names() {
        assert_eq!(service_name(80), Some("http"));
        assert_eq!(service_name(443), Some("https"));
        assert_eq!(service_name(22), Some("ssh"));
        assert_eq!(service_name(53), Some("dns"));
        assert_eq!(service_name(12345), None);
    }

    #[test]
    fn test_tcp_flags() {
        // SYN-ACK
        let data = vec![
            0x00, 0x50, 0xc0, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
            0x50, 0x12, // SYN + ACK
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let info = parse_tcp(&data).unwrap();
        let flags = info.tcp_flags.unwrap();

        assert!(flags.syn);
        assert!(flags.ack);
        assert!(flags.is_syn_ack());
        assert!(!flags.is_syn_only());
    }
}
