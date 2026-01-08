//! System audio actuators

use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetVolumeParams {
    #[schemars(description = "Volume level (0-100)")]
    pub volume: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetMuteParams {
    #[schemars(description = "Whether to mute (true) or unmute (false)")]
    pub muted: bool,
}

// === Tool Functions ===

pub async fn get_volume() -> Result<CallToolResult, McpError> {
    let volume = cpvc::get_system_volume();
    Ok(CallToolResult::success(vec![Content::text(format!(
        "{}",
        volume
    ))]))
}

pub async fn set_volume(params: SetVolumeParams) -> Result<CallToolResult, McpError> {
    let volume = params.volume.min(100);
    let success = cpvc::set_system_volume(volume);
    if success {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Volume set to {}%",
            volume
        ))]))
    } else {
        Ok(CallToolResult::success(vec![Content::text(
            "Failed to set volume",
        )]))
    }
}

pub async fn get_mute() -> Result<CallToolResult, McpError> {
    let muted = cpvc::get_mute();
    Ok(CallToolResult::success(vec![Content::text(
        if muted { "muted" } else { "unmuted" },
    )]))
}

pub async fn set_mute(params: SetMuteParams) -> Result<CallToolResult, McpError> {
    let success = cpvc::set_mute(params.muted);
    if success {
        Ok(CallToolResult::success(vec![Content::text(
            if params.muted {
                "Audio muted"
            } else {
                "Audio unmuted"
            },
        )]))
    } else {
        Ok(CallToolResult::success(vec![Content::text(
            "Failed to change mute status",
        )]))
    }
}

pub async fn list_audio_devices() -> Result<CallToolResult, McpError> {
    let devices = cpvc::get_sound_devices();
    if devices.is_empty() {
        Ok(CallToolResult::success(vec![Content::text(
            "No audio devices found",
        )]))
    } else {
        let list = devices.join("\n");
        Ok(CallToolResult::success(vec![Content::text(list)]))
    }
}
