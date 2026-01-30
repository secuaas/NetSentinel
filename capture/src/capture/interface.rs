//! Network interface management

use anyhow::{Context, Result, bail};
use pnet::datalink::{self, NetworkInterface as PnetInterface};
use std::net::IpAddr;
use tracing::{info, warn};

/// Represents a network interface
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    /// Interface name (e.g., "eth0")
    pub name: String,

    /// Interface index
    pub index: u32,

    /// MAC address (if available)
    pub mac: Option<[u8; 6]>,

    /// IP addresses assigned to this interface
    pub ips: Vec<IpAddr>,

    /// Whether the interface is up
    pub is_up: bool,

    /// Whether the interface is a loopback
    pub is_loopback: bool,

    /// MTU (if available)
    pub mtu: Option<u32>,
}

impl NetworkInterface {
    /// Get a network interface by name
    pub fn by_name(name: &str) -> Result<Self> {
        let interfaces = datalink::interfaces();

        let iface = interfaces
            .into_iter()
            .find(|i| i.name == name)
            .with_context(|| format!("Interface '{}' not found", name))?;

        Self::from_pnet(iface)
    }

    /// Get all available network interfaces
    pub fn list_all() -> Vec<Self> {
        datalink::interfaces()
            .into_iter()
            .filter_map(|i| Self::from_pnet(i).ok())
            .collect()
    }

    /// Get interfaces suitable for monitoring (non-loopback, up)
    pub fn list_monitoring() -> Vec<Self> {
        Self::list_all()
            .into_iter()
            .filter(|i| i.is_up && !i.is_loopback)
            .collect()
    }

    /// Convert from pnet NetworkInterface
    fn from_pnet(iface: PnetInterface) -> Result<Self> {
        let mac = iface.mac.map(|m| m.octets());
        let ips: Vec<IpAddr> = iface.ips.iter().map(|ip| ip.ip()).collect();

        // Extract values before moving name
        let is_up = iface.is_up();
        let is_loopback = iface.is_loopback();
        let index = iface.index;

        Ok(Self {
            name: iface.name,
            index,
            mac,
            ips,
            is_up,
            is_loopback,
            mtu: None, // pnet doesn't expose MTU directly
        })
    }

    /// Check if the interface is valid for capture
    pub fn validate_for_capture(&self) -> Result<()> {
        if !self.is_up {
            bail!("Interface '{}' is not up", self.name);
        }

        if self.is_loopback {
            warn!("Interface '{}' is a loopback interface", self.name);
        }

        info!(
            "Interface '{}' validated: MAC={}, IPs={:?}",
            self.name,
            self.mac
                .map(|m| format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    m[0], m[1], m[2], m[3], m[4], m[5]
                ))
                .unwrap_or_else(|| "unknown".to_string()),
            self.ips
        );

        Ok(())
    }

    /// Set interface to promiscuous mode using ioctl
    #[cfg(target_os = "linux")]
    pub fn set_promiscuous(&self, enable: bool) -> Result<()> {
        use std::ffi::CString;
        use libc::{c_int, c_short, ioctl, socket, AF_INET, IFF_PROMISC, SIOCGIFFLAGS, SIOCSIFFLAGS, SOCK_DGRAM};
        use std::mem::zeroed;

        // ifreq structure
        #[repr(C)]
        struct ifreq {
            ifr_name: [libc::c_char; 16],
            ifr_flags: c_short,
        }

        unsafe {
            let sock = socket(AF_INET, SOCK_DGRAM, 0);
            if sock < 0 {
                bail!("Failed to create socket for ioctl");
            }

            let ifname = CString::new(self.name.as_str())?;
            let mut req: ifreq = zeroed();

            // Copy interface name
            let name_bytes = ifname.as_bytes_with_nul();
            for (i, &b) in name_bytes.iter().take(15).enumerate() {
                req.ifr_name[i] = b as libc::c_char;
            }

            // Get current flags
            if ioctl(sock, SIOCGIFFLAGS as libc::c_ulong, &mut req as *mut ifreq) < 0 {
                libc::close(sock);
                bail!("Failed to get interface flags for '{}'", self.name);
            }

            // Modify promiscuous flag
            if enable {
                req.ifr_flags |= IFF_PROMISC as c_short;
            } else {
                req.ifr_flags &= !(IFF_PROMISC as c_short);
            }

            // Set new flags
            if ioctl(sock, SIOCSIFFLAGS as libc::c_ulong, &req as *const ifreq) < 0 {
                libc::close(sock);
                bail!("Failed to set promiscuous mode on '{}'. Are you running as root?", self.name);
            }

            libc::close(sock);
        }

        info!(
            "Promiscuous mode {} on interface '{}'",
            if enable { "enabled" } else { "disabled" },
            self.name
        );

        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn set_promiscuous(&self, enable: bool) -> Result<()> {
        warn!(
            "Promiscuous mode control not implemented for this platform. Interface: {}, requested: {}",
            self.name, enable
        );
        Ok(())
    }
}

/// Print information about all interfaces
pub fn print_interfaces() {
    println!("Available network interfaces:");
    println!("{:-<60}", "");

    for iface in NetworkInterface::list_all() {
        let mac_str = iface
            .mac
            .map(|m| {
                format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    m[0], m[1], m[2], m[3], m[4], m[5]
                )
            })
            .unwrap_or_else(|| "N/A".to_string());

        let status = if iface.is_up { "UP" } else { "DOWN" };
        let loopback = if iface.is_loopback { " (loopback)" } else { "" };

        println!(
            "{}: {} [{}{}]",
            iface.name, mac_str, status, loopback
        );

        for ip in &iface.ips {
            println!("    {}", ip);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_interfaces() {
        let interfaces = NetworkInterface::list_all();
        assert!(!interfaces.is_empty(), "Should have at least one interface");

        // Should have loopback
        assert!(
            interfaces.iter().any(|i| i.is_loopback),
            "Should have a loopback interface"
        );
    }

    #[test]
    fn test_interface_by_name() {
        // loopback should exist on all systems
        let lo = NetworkInterface::by_name("lo");
        if lo.is_ok() {
            let lo = lo.unwrap();
            assert!(lo.is_loopback);
            assert!(lo.is_up);
        }
    }
}
