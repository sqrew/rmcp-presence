//! USB device sensors

use crate::shared::internal_error;
use nusb::list_devices;
use rmcp::{model::*, ErrorData as McpError};

// === Tool Functions ===

pub async fn get_usb_devices() -> Result<CallToolResult, McpError> {
    let devices =
        list_devices().map_err(|e| internal_error(format!("Failed to list USB devices: {}", e)))?;

    let mut result = String::from("USB Devices:\n\n");
    let mut count = 0;

    for device in devices {
        count += 1;

        let manufacturer = device.manufacturer_string().unwrap_or_default();
        let product = device.product_string().unwrap_or_default();
        let serial = device.serial_number().unwrap_or_default();

        let display_name = if !product.is_empty() {
            product.to_string()
        } else {
            format!(
                "Device {:04x}:{:04x}",
                device.vendor_id(),
                device.product_id()
            )
        };

        result.push_str(&format!("{}. {}\n", count, display_name));

        if !manufacturer.is_empty() {
            result.push_str(&format!("   Manufacturer: {}\n", manufacturer));
        }

        result.push_str(&format!(
            "   Vendor ID: {:04x}, Product ID: {:04x}\n",
            device.vendor_id(),
            device.product_id()
        ));

        if !serial.is_empty() {
            result.push_str(&format!("   Serial: {}\n", serial));
        }

        result.push_str(&format!(
            "   Bus: {}, Device: {}\n\n",
            device.bus_number(),
            device.device_address()
        ));
    }

    if count == 0 {
        result.push_str("No USB devices found.\n");
    } else {
        result.push_str(&format!("Total: {} USB devices\n", count));
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
