//! Network interface sensors

use crate::shared::internal_error;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use rmcp::{model::*, ErrorData as McpError};

// === Tool Functions ===

pub async fn get_interfaces() -> Result<CallToolResult, McpError> {
    let interfaces = NetworkInterface::show()
        .map_err(|e| internal_error(format!("Failed to get network interfaces: {}", e)))?;

    let mut result = String::from("Network Interfaces:\n\n");

    if interfaces.is_empty() {
        result.push_str("No network interfaces found.\n");
    } else {
        for iface in &interfaces {
            let is_loopback = iface.addr.iter().any(|a| match a {
                Addr::V4(v4) => v4.ip.is_loopback(),
                Addr::V6(v6) => v6.ip.is_loopback(),
            });

            result.push_str(&format!("{}", iface.name));
            if is_loopback {
                result.push_str(" (loopback)");
            }
            result.push('\n');

            if let Some(ref mac) = iface.mac_addr {
                if !mac.is_empty() && mac != "00:00:00:00:00:00" {
                    result.push_str(&format!("  MAC: {}\n", mac));
                }
            }

            for addr in &iface.addr {
                match addr {
                    Addr::V4(v4) => {
                        result.push_str(&format!("  IPv4: {}", v4.ip));
                        if let Some(netmask) = &v4.netmask {
                            result.push_str(&format!(" / {}", netmask));
                        }
                        result.push('\n');
                    }
                    Addr::V6(v6) => {
                        if !v6.ip.to_string().starts_with("fe80") {
                            result.push_str(&format!("  IPv6: {}\n", v6.ip));
                        }
                    }
                }
            }
            result.push('\n');
        }

        let active_count = interfaces.iter().filter(|i| !i.addr.is_empty()).count();
        result.push_str(&format!(
            "Total interfaces: {} ({} with addresses)\n",
            interfaces.len(),
            active_count
        ));
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
