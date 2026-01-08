//! Bluetooth Low Energy (BLE) sensors

use crate::shared::internal_error;
use btleplug::api::{Central, Manager as BtManager, Peripheral as _, ScanFilter};
use btleplug::platform::Manager as BluetoothManager;
use rmcp::{model::*, ErrorData as McpError};
use std::time::Duration;

// === Tool Functions ===

pub async fn scan_ble_devices() -> Result<CallToolResult, McpError> {
    let manager = BluetoothManager::new()
        .await
        .map_err(|e| internal_error(format!("Failed to create BT manager: {}", e)))?;

    let adapters = manager
        .adapters()
        .await
        .map_err(|e| internal_error(format!("Failed to get adapters: {}", e)))?;

    if adapters.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "Bluetooth Status:\n\nNo Bluetooth adapters found.\n",
        )]));
    }

    let mut result = String::from("Bluetooth Devices:\n\n");

    for adapter in adapters {
        let adapter_info = adapter
            .adapter_info()
            .await
            .unwrap_or_else(|_| "Unknown adapter".to_string());
        result.push_str(&format!("Adapter: {}\n\n", adapter_info));

        if let Err(e) = adapter.start_scan(ScanFilter::default()).await {
            result.push_str(&format!("  Could not scan: {}\n", e));
            continue;
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
        let _ = adapter.stop_scan().await;

        let peripherals = adapter
            .peripherals()
            .await
            .map_err(|e| internal_error(format!("Failed to get peripherals: {}", e)))?;

        if peripherals.is_empty() {
            result.push_str("  No BLE devices found nearby.\n");
        } else {
            let mut count = 0;
            for peripheral in peripherals {
                count += 1;

                let properties = peripheral.properties().await.ok().flatten();

                let name = properties
                    .as_ref()
                    .and_then(|p| p.local_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                let address = properties
                    .as_ref()
                    .map(|p| p.address.to_string())
                    .unwrap_or_else(|| "??:??:??:??:??:??".to_string());

                let rssi = properties
                    .as_ref()
                    .and_then(|p| p.rssi)
                    .map(|r| format!(" ({}dBm)", r))
                    .unwrap_or_default();

                result.push_str(&format!("  {}. {}{}\n", count, name, rssi));
                result.push_str(&format!("     Address: {}\n", address));
            }
            result.push_str(&format!("\n  Total: {} BLE devices\n", count));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
