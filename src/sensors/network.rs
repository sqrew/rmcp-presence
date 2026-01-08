//! Network interface sensors

use crate::shared::internal_error;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use rmcp::{model::*, ErrorData as McpError};
use serde::{Deserialize, Serialize};

/// Response from ipinfo.io
#[derive(Debug, Deserialize, Serialize)]
pub struct IpInfo {
    pub ip: String,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub loc: Option<String>,
    #[serde(default)]
    pub org: Option<String>,
    #[serde(default)]
    pub postal: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
}

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

/// Get public IP address and geolocation info from ipinfo.io
pub async fn get_public_ip() -> Result<CallToolResult, McpError> {
    let client = reqwest::Client::new();

    let info: IpInfo = client
        .get("https://ipinfo.io/json")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| internal_error(format!("Failed to fetch IP info: {}", e)))?
        .json()
        .await
        .map_err(|e| internal_error(format!("Failed to parse IP info: {}", e)))?;

    let json = serde_json::to_string_pretty(&info)
        .map_err(|e| internal_error(format!("Serialization error: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}
