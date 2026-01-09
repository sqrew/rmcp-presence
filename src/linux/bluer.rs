//! Bluetooth control via BlueZ

use bluer::{Adapter, AdapterEvent, Address, Session};
use futures::{pin_mut, StreamExt};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AdapterParams {
    #[schemars(description = "Adapter name (e.g., \"hci0\"). Uses default if not specified.")]
    pub adapter: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiscoverParams {
    #[schemars(description = "Adapter name (e.g., \"hci0\"). Uses default if not specified.")]
    pub adapter: Option<String>,
    #[schemars(description = "Discovery duration in seconds (default: 10, max: 30)")]
    pub duration: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeviceParams {
    #[schemars(description = "Bluetooth address of the device (e.g., \"AA:BB:CC:DD:EE:FF\")")]
    pub address: String,
    #[schemars(description = "Adapter name (e.g., \"hci0\"). Uses default if not specified.")]
    pub adapter: Option<String>,
}

// === Helper Functions ===

async fn get_session() -> Result<Session, String> {
    Session::new()
        .await
        .map_err(|e| format!("Failed to connect to BlueZ: {}", e))
}

async fn get_adapter(session: &Session, name: Option<&str>) -> Result<Adapter, String> {
    match name {
        Some(n) => session
            .adapter(n)
            .map_err(|e| format!("Failed to get adapter '{}': {}", n, e)),
        None => session
            .default_adapter()
            .await
            .map_err(|e| format!("Failed to get default adapter: {}", e)),
    }
}

fn parse_address(addr: &str) -> Result<Address, String> {
    addr.parse()
        .map_err(|_| format!("Invalid Bluetooth address: {}", addr))
}

// === Tool Functions ===

pub async fn list_adapters() -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let names = match session.adapter_names().await {
        Ok(n) => n,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list adapters: {}",
                e
            ))]))
        }
    };

    if names.is_empty() {
        Ok(CallToolResult::success(vec![Content::text(
            "No Bluetooth adapters found",
        )]))
    } else {
        let mut output = format!("{} Bluetooth adapter(s):\n", names.len());
        for name in &names {
            output.push_str(&format!("  - {}\n", name));
        }
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }
}

pub async fn get_adapter_info(params: AdapterParams) -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let adapter = match get_adapter(&session, params.adapter.as_deref()).await {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let name = adapter.name().to_string();
    let address = adapter
        .address()
        .await
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".into());
    let powered = adapter.is_powered().await.unwrap_or(false);
    let discoverable = adapter.is_discoverable().await.unwrap_or(false);
    let pairable = adapter.is_pairable().await.unwrap_or(false);
    let discovering = adapter.is_discovering().await.unwrap_or(false);
    let alias = adapter.alias().await.unwrap_or_else(|_| "unknown".into());

    let output = format!(
        "Adapter: {}\n\
         Address: {}\n\
         Alias: {}\n\
         Powered: {}\n\
         Discoverable: {}\n\
         Pairable: {}\n\
         Discovering: {}",
        name, address, alias, powered, discoverable, pairable, discovering
    );

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn discover_devices(params: DiscoverParams) -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let adapter = match get_adapter(&session, params.adapter.as_deref()).await {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    if !adapter.is_powered().await.unwrap_or(false) {
        return Ok(CallToolResult::success(vec![Content::text(
            "Adapter is not powered on. Cannot discover devices.",
        )]));
    }

    let duration = params.duration.unwrap_or(10).min(30);
    let timeout = Duration::from_secs(duration);

    let discover = match adapter.discover_devices().await {
        Ok(d) => d,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to start discovery: {}",
                e
            ))]))
        }
    };

    pin_mut!(discover);

    let mut devices: Vec<(Address, Option<String>, Option<i16>)> = Vec::new();
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() >= timeout {
            break;
        }

        let remaining = timeout.saturating_sub(start.elapsed());

        match tokio::time::timeout(remaining, discover.next()).await {
            Ok(Some(AdapterEvent::DeviceAdded(addr))) => {
                if let Ok(device) = adapter.device(addr) {
                    let name = device.name().await.ok().flatten();
                    let rssi = device.rssi().await.ok().flatten();

                    if !devices.iter().any(|(a, _, _)| *a == addr) {
                        devices.push((addr, name, rssi));
                    }
                }
            }
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => break,
        }
    }

    if devices.is_empty() {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "No devices found in {} seconds",
            duration
        ))]))
    } else {
        let mut output = format!(
            "Found {} device(s) in {} seconds:\n",
            devices.len(),
            duration
        );
        for (addr, name, rssi) in &devices {
            let name_str = name.as_deref().unwrap_or("(unknown)");
            let rssi_str = rssi
                .map(|r| format!(" [RSSI: {} dBm]", r))
                .unwrap_or_default();
            output.push_str(&format!("  {} - {}{}\n", addr, name_str, rssi_str));
        }
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }
}

pub async fn list_known_devices(params: AdapterParams) -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let adapter = match get_adapter(&session, params.adapter.as_deref()).await {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let addresses = match adapter.device_addresses().await {
        Ok(a) => a,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list devices: {}",
                e
            ))]))
        }
    };

    if addresses.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No known devices",
        )]));
    }

    let mut output = format!("{} known device(s):\n", addresses.len());

    for addr in addresses {
        if let Ok(device) = adapter.device(addr) {
            let name = device
                .name()
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "(unknown)".into());
            let paired = device.is_paired().await.unwrap_or(false);
            let connected = device.is_connected().await.unwrap_or(false);

            let status = match (paired, connected) {
                (true, true) => "[paired, connected]",
                (true, false) => "[paired]",
                (false, true) => "[connected]",
                (false, false) => "",
            };

            output.push_str(&format!("  {} - {} {}\n", addr, name, status));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn get_device_info(params: DeviceParams) -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let adapter = match get_adapter(&session, params.adapter.as_deref()).await {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let address = match parse_address(&params.address) {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let device = match adapter.device(address) {
        Ok(d) => d,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Device not found: {}",
                e
            ))]))
        }
    };

    let name = device
        .name()
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "(unknown)".into());
    let alias = device.alias().await.unwrap_or_else(|_| "(unknown)".into());
    let paired = device.is_paired().await.unwrap_or(false);
    let connected = device.is_connected().await.unwrap_or(false);
    let trusted = device.is_trusted().await.unwrap_or(false);
    let blocked = device.is_blocked().await.unwrap_or(false);
    let rssi = device.rssi().await.ok().flatten();
    let tx_power = device.tx_power().await.ok().flatten();
    let uuids = device.uuids().await.ok().flatten().unwrap_or_default();
    let addr_type = device.address_type().await.ok();

    let mut output = format!(
        "Device: {}\n\
         Name: {}\n\
         Alias: {}\n\
         Address Type: {:?}\n\
         Paired: {}\n\
         Connected: {}\n\
         Trusted: {}\n\
         Blocked: {}",
        address, name, alias, addr_type, paired, connected, trusted, blocked
    );

    if let Some(r) = rssi {
        output.push_str(&format!("\nRSSI: {} dBm", r));
    }
    if let Some(tx) = tx_power {
        output.push_str(&format!("\nTX Power: {} dBm", tx));
    }
    if !uuids.is_empty() {
        output.push_str(&format!("\nServices: {} UUID(s)", uuids.len()));
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn pair_device(params: DeviceParams) -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let adapter = match get_adapter(&session, params.adapter.as_deref()).await {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let address = match parse_address(&params.address) {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let device = match adapter.device(address) {
        Ok(d) => d,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Device not found: {}. Run discover_devices first.",
                e
            ))]))
        }
    };

    if device.is_paired().await.unwrap_or(false) {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "Device {} is already paired",
            address
        ))]));
    }

    match device.pair().await {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Successfully paired with {}",
            address
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to pair with {}: {}",
            address, e
        ))])),
    }
}

pub async fn remove_device(params: DeviceParams) -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let adapter = match get_adapter(&session, params.adapter.as_deref()).await {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let address = match parse_address(&params.address) {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match adapter.remove_device(address).await {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Removed device {}",
            address
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to remove {}: {}",
            address, e
        ))])),
    }
}

pub async fn connect_device(params: DeviceParams) -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let adapter = match get_adapter(&session, params.adapter.as_deref()).await {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let address = match parse_address(&params.address) {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let device = match adapter.device(address) {
        Ok(d) => d,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Device not found: {}",
                e
            ))]))
        }
    };

    if device.is_connected().await.unwrap_or(false) {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "Device {} is already connected",
            address
        ))]));
    }

    match device.connect().await {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Connected to {}",
            address
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to connect to {}: {}",
            address, e
        ))])),
    }
}

// === Composite Types ===

#[derive(Debug, serde::Serialize)]
pub struct BluetoothStatus {
    pub adapter: Option<AdapterStatus>,
    pub devices: Vec<DeviceStatus>,
    pub connected_count: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct AdapterStatus {
    pub name: String,
    pub address: String,
    pub powered: bool,
    pub discoverable: bool,
    pub discovering: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct DeviceStatus {
    pub address: String,
    pub name: String,
    pub paired: bool,
    pub connected: bool,
    pub trusted: bool,
}

// === Composite Function ===

pub async fn get_bluetooth_status(params: AdapterParams) -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(format!(
            "{{\"error\": \"{}\"}}",
            e
        ))])),
    };

    let adapter = match get_adapter(&session, params.adapter.as_deref()).await {
        Ok(a) => a,
        Err(e) => {
            // No adapter available - return empty status
            let status = BluetoothStatus {
                adapter: None,
                devices: vec![],
                connected_count: 0,
            };
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&status).unwrap_or_else(|_| e),
            )]));
        }
    };

    // Build adapter info
    let adapter_status = AdapterStatus {
        name: adapter.name().to_string(),
        address: adapter
            .address()
            .await
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".into()),
        powered: adapter.is_powered().await.unwrap_or(false),
        discoverable: adapter.is_discoverable().await.unwrap_or(false),
        discovering: adapter.is_discovering().await.unwrap_or(false),
    };

    // Build device list
    let mut devices = Vec::new();
    let mut connected_count = 0;

    if let Ok(addresses) = adapter.device_addresses().await {
        for addr in addresses {
            if let Ok(device) = adapter.device(addr) {
                let connected = device.is_connected().await.unwrap_or(false);
                if connected {
                    connected_count += 1;
                }

                devices.push(DeviceStatus {
                    address: addr.to_string(),
                    name: device
                        .name()
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "(unknown)".into()),
                    paired: device.is_paired().await.unwrap_or(false),
                    connected,
                    trusted: device.is_trusted().await.unwrap_or(false),
                });
            }
        }
    }

    let status = BluetoothStatus {
        adapter: Some(adapter_status),
        devices,
        connected_count,
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&status).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
    )]))
}

pub async fn disconnect_device(params: DeviceParams) -> Result<CallToolResult, McpError> {
    let session = match get_session().await {
        Ok(s) => s,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let adapter = match get_adapter(&session, params.adapter.as_deref()).await {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let address = match parse_address(&params.address) {
        Ok(a) => a,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let device = match adapter.device(address) {
        Ok(d) => d,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Device not found: {}",
                e
            ))]))
        }
    };

    if !device.is_connected().await.unwrap_or(false) {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "Device {} is not connected",
            address
        ))]));
    }

    match device.disconnect().await {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Disconnected from {}",
            address
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to disconnect from {}: {}",
            address, e
        ))])),
    }
}
