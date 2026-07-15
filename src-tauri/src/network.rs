use serde::Serialize;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub address: String,
}

/// Gather all valid IPv4 non-loopback network addresses on the machine.
pub fn get_all_ip_addresses() -> Vec<NetworkInterfaceInfo> {
    let mut address_list = Vec::new();

    if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
        for (name, ip) in interfaces {
            if let IpAddr::V4(v4) = ip {
                if !v4.is_loopback() {
                    address_list.push(NetworkInterfaceInfo {
                        name,
                        address: v4.to_string(),
                    });
                }
            }
        }
    }

    // Fallback to localhost if no active network interfaces are found
    if address_list.is_empty() {
        address_list.push(NetworkInterfaceInfo {
            name: "Local Loopback".to_string(),
            address: "127.0.0.1".to_string(),
        });
    }

    address_list
}
