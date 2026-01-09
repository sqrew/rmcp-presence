//! Network interface sensors

use crate::shared::internal_error;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

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

// === Params ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DnsLookupParams {
    /// Hostname to resolve (e.g., "google.com")
    pub hostname: String,
}

// === New Tools ===

/// Check if we have internet connectivity
pub async fn is_online() -> Result<CallToolResult, McpError> {
    // Try to connect to Cloudflare DNS (1.1.1.1:53) with a 3 second timeout
    let result = tokio::task::spawn_blocking(|| {
        TcpStream::connect_timeout(
            &"1.1.1.1:53".parse().unwrap(),
            Duration::from_secs(3),
        )
    })
    .await
    .map_err(|e| internal_error(format!("Task error: {}", e)))?;

    let online = result.is_ok();
    let json = serde_json::json!({
        "online": online,
        "checked_host": "1.1.1.1:53",
        "method": "tcp_connect"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap(),
    )]))
}

/// Resolve a hostname to IP addresses
pub async fn dns_lookup(params: DnsLookupParams) -> Result<CallToolResult, McpError> {
    let hostname = params.hostname.clone();

    let result = tokio::task::spawn_blocking(move || {
        let lookup = format!("{}:0", hostname);
        lookup.to_socket_addrs()
    })
    .await
    .map_err(|e| internal_error(format!("Task error: {}", e)))?;

    match result {
        Ok(addrs) => {
            let ips: Vec<String> = addrs
                .map(|addr| addr.ip().to_string())
                .collect();

            let json = serde_json::json!({
                "hostname": params.hostname,
                "resolved": true,
                "addresses": ips
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json).unwrap(),
            )]))
        }
        Err(e) => {
            let json = serde_json::json!({
                "hostname": params.hostname,
                "resolved": false,
                "error": e.to_string()
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json).unwrap(),
            )]))
        }
    }
}

/// Comprehensive network status - everything at once
#[derive(Debug, Serialize)]
pub struct NetworkInfo {
    pub online: bool,
    pub public_ip: Option<String>,
    pub location: Option<NetworkLocation>,
    pub interfaces: Vec<InterfaceInfo>,
    pub stats: NetworkStats,
}

#[derive(Debug, Serialize)]
pub struct NetworkLocation {
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub mac: Option<String>,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub is_loopback: bool,
}

#[derive(Debug, Serialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

pub async fn get_network_info() -> Result<CallToolResult, McpError> {
    // Check online status
    let online = tokio::task::spawn_blocking(|| {
        TcpStream::connect_timeout(
            &"1.1.1.1:53".parse().unwrap(),
            Duration::from_secs(3),
        ).is_ok()
    })
    .await
    .unwrap_or(false);

    // Get public IP if online
    let (public_ip, location) = if online {
        let client = reqwest::Client::new();
        match client
            .get("https://ipinfo.io/json")
            .header("Accept", "application/json")
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => match resp.json::<IpInfo>().await {
                Ok(info) => (
                    Some(info.ip),
                    Some(NetworkLocation {
                        city: info.city,
                        region: info.region,
                        country: info.country,
                        timezone: info.timezone,
                    }),
                ),
                Err(_) => (None, None),
            },
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    // Get interfaces
    let interfaces = NetworkInterface::show()
        .unwrap_or_default()
        .into_iter()
        .filter(|iface| !iface.addr.is_empty())
        .map(|iface| {
            let is_loopback = iface.addr.iter().any(|a| match a {
                Addr::V4(v4) => v4.ip.is_loopback(),
                Addr::V6(v6) => v6.ip.is_loopback(),
            });

            let ipv4: Vec<String> = iface
                .addr
                .iter()
                .filter_map(|a| match a {
                    Addr::V4(v4) => Some(v4.ip.to_string()),
                    _ => None,
                })
                .collect();

            let ipv6: Vec<String> = iface
                .addr
                .iter()
                .filter_map(|a| match a {
                    Addr::V6(v6) if !v6.ip.to_string().starts_with("fe80") => {
                        Some(v6.ip.to_string())
                    }
                    _ => None,
                })
                .collect();

            InterfaceInfo {
                name: iface.name,
                mac: iface.mac_addr.filter(|m| !m.is_empty() && m != "00:00:00:00:00:00"),
                ipv4,
                ipv6,
                is_loopback,
            }
        })
        .collect();

    // Get network stats
    let networks = sysinfo::Networks::new_with_refreshed_list();

    let (bytes_sent, bytes_received) = networks
        .iter()
        .fold((0u64, 0u64), |(sent, recv), (_, data)| {
            (sent + data.total_transmitted(), recv + data.total_received())
        });

    let info = NetworkInfo {
        online,
        public_ip,
        location,
        interfaces,
        stats: NetworkStats {
            bytes_sent,
            bytes_received,
        },
    };

    let json = serde_json::to_string_pretty(&info)
        .map_err(|e| internal_error(format!("Serialization error: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}
