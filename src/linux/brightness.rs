//! Screen brightness control

use brightness::{brightness_devices, Brightness};
use futures::TryStreamExt;
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeviceParams {
    #[schemars(description = "Device name (optional - uses first device if not specified)")]
    pub device: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetBrightnessParams {
    #[schemars(description = "Brightness level (0-100 percent)")]
    pub brightness: u32,
    #[schemars(description = "Device name (optional - uses first device if not specified)")]
    pub device: Option<String>,
}

// === Tool Functions ===

pub async fn list_brightness_devices() -> Result<CallToolResult, McpError> {
    let devices: Vec<String> = match brightness_devices()
        .try_filter_map(|dev| async move {
            match dev.device_name().await {
                Ok(name) => Ok(Some(name)),
                Err(_) => Ok(None),
            }
        })
        .try_collect()
        .await
    {
        Ok(d) => d,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list devices: {}",
                e
            ))]))
        }
    };

    if devices.is_empty() {
        Ok(CallToolResult::success(vec![Content::text(
            "No brightness devices found",
        )]))
    } else {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{} brightness device(s):\n{}",
            devices.len(),
            devices.join("\n")
        ))]))
    }
}

pub async fn get_brightness(params: DeviceParams) -> Result<CallToolResult, McpError> {
    let mut devices = brightness_devices();

    while let Ok(Some(dev)) = devices.try_next().await {
        let name = match dev.device_name().await {
            Ok(n) => n,
            Err(_) => continue,
        };

        if let Some(ref target) = params.device {
            if !name.to_lowercase().contains(&target.to_lowercase()) {
                continue;
            }
        }

        match dev.get().await {
            Ok(brightness) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "{}: {}%",
                    name, brightness
                ))]))
            }
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Failed to get brightness for {}: {}",
                    name, e
                ))]))
            }
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        match params.device {
            Some(d) => format!("Device '{}' not found", d),
            None => "No brightness devices found".to_string(),
        },
    )]))
}

pub async fn set_brightness(params: SetBrightnessParams) -> Result<CallToolResult, McpError> {
    let brightness = params.brightness.min(100);
    let mut devices = brightness_devices();

    while let Ok(Some(mut dev)) = devices.try_next().await {
        let name = match dev.device_name().await {
            Ok(n) => n,
            Err(_) => continue,
        };

        if let Some(ref target) = params.device {
            if !name.to_lowercase().contains(&target.to_lowercase()) {
                continue;
            }
        }

        match dev.set(brightness).await {
            Ok(()) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "{}: brightness set to {}%",
                    name, brightness
                ))]))
            }
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Failed to set brightness for {}: {}",
                    name, e
                ))]))
            }
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        match params.device {
            Some(d) => format!("Device '{}' not found", d),
            None => "No brightness devices found".to_string(),
        },
    )]))
}
