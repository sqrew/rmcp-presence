//! PulseAudio per-app volume control

use crate::shared::internal_error;
use pulsectl::controllers::{AppControl, DeviceControl, SinkController, SourceController};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NameParams {
    #[schemars(description = "Device name")]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct VolumeParams {
    #[schemars(description = "Application index")]
    pub index: u32,
    #[schemars(description = "Volume change in percent (positive to increase, negative to decrease)")]
    pub delta: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MuteParams {
    #[schemars(description = "Application index")]
    pub index: u32,
    #[schemars(description = "Whether to mute (true) or unmute (false)")]
    pub mute: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveAppParams {
    #[schemars(description = "Application/stream index")]
    pub app_index: u32,
    #[schemars(description = "Target device name to move the app to")]
    pub device_name: String,
}

// === Tool Functions ===

pub async fn list_sinks() -> Result<CallToolResult, McpError> {
    let result = tokio::task::spawn_blocking(|| {
        let mut handler = SinkController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        let devices = handler
            .list_devices()
            .map_err(|e| format!("Failed to list sinks: {:?}", e))?;

        if devices.is_empty() {
            return Ok("No sinks found".to_string());
        }

        let mut output = format!("{} sink(s):\n\n", devices.len());
        for dev in devices {
            let vol_percent = dev
                .volume
                .get()
                .first()
                .map(|v| v.0 as f64 / 65536.0 * 100.0)
                .unwrap_or(0.0);
            output.push_str(&format!(
                "  [{}] {} - {}\n      Volume: {:.0}%, Muted: {}\n",
                dev.index,
                dev.name.as_deref().unwrap_or("unknown"),
                dev.description.as_deref().unwrap_or(""),
                vol_percent,
                dev.mute
            ));
        }
        Ok::<_, String>(output)
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn list_sources() -> Result<CallToolResult, McpError> {
    let result = tokio::task::spawn_blocking(|| {
        let mut handler = SourceController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        let devices = handler
            .list_devices()
            .map_err(|e| format!("Failed to list sources: {:?}", e))?;

        if devices.is_empty() {
            return Ok("No sources found".to_string());
        }

        let mut output = format!("{} source(s):\n\n", devices.len());
        for dev in devices {
            let vol_percent = dev
                .volume
                .get()
                .first()
                .map(|v| v.0 as f64 / 65536.0 * 100.0)
                .unwrap_or(0.0);
            output.push_str(&format!(
                "  [{}] {} - {}\n      Volume: {:.0}%, Muted: {}\n",
                dev.index,
                dev.name.as_deref().unwrap_or("unknown"),
                dev.description.as_deref().unwrap_or(""),
                vol_percent,
                dev.mute
            ));
        }
        Ok::<_, String>(output)
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn list_sink_inputs() -> Result<CallToolResult, McpError> {
    let result = tokio::task::spawn_blocking(|| {
        let mut handler = SinkController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        let apps = handler
            .list_applications()
            .map_err(|e| format!("Failed to list sink inputs: {:?}", e))?;

        if apps.is_empty() {
            return Ok("No applications playing audio".to_string());
        }

        let mut output = format!("{} application(s) playing audio:\n\n", apps.len());
        for app in apps {
            let vol_percent = app
                .volume
                .get()
                .first()
                .map(|v| v.0 as f64 / 65536.0 * 100.0)
                .unwrap_or(0.0);
            let app_name = app.proplist.get_str("application.name").unwrap_or_default();
            output.push_str(&format!(
                "  [{}] {} ({})\n      Volume: {:.0}%, Muted: {}\n",
                app.index,
                app.name.as_deref().unwrap_or("unknown"),
                app_name,
                vol_percent,
                app.mute
            ));
        }
        Ok::<_, String>(output)
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn list_source_outputs() -> Result<CallToolResult, McpError> {
    let result = tokio::task::spawn_blocking(|| {
        let mut handler = SourceController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        let apps = handler
            .list_applications()
            .map_err(|e| format!("Failed to list source outputs: {:?}", e))?;

        if apps.is_empty() {
            return Ok("No applications recording audio".to_string());
        }

        let mut output = format!("{} application(s) recording audio:\n\n", apps.len());
        for app in apps {
            let vol_percent = app
                .volume
                .get()
                .first()
                .map(|v| v.0 as f64 / 65536.0 * 100.0)
                .unwrap_or(0.0);
            let app_name = app.proplist.get_str("application.name").unwrap_or_default();
            output.push_str(&format!(
                "  [{}] {} ({})\n      Volume: {:.0}%, Muted: {}\n",
                app.index,
                app.name.as_deref().unwrap_or("unknown"),
                app_name,
                vol_percent,
                app.mute
            ));
        }
        Ok::<_, String>(output)
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn get_default_sink() -> Result<CallToolResult, McpError> {
    let result = tokio::task::spawn_blocking(|| {
        let mut handler = SinkController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        let dev = handler
            .get_default_device()
            .map_err(|e| format!("Failed to get default sink: {:?}", e))?;

        let vol_percent = dev
            .volume
            .get()
            .first()
            .map(|v| v.0 as f64 / 65536.0 * 100.0)
            .unwrap_or(0.0);
        Ok::<_, String>(format!(
            "Default sink:\n  [{}] {} - {}\n  Volume: {:.0}%, Muted: {}",
            dev.index,
            dev.name.as_deref().unwrap_or("unknown"),
            dev.description.as_deref().unwrap_or(""),
            vol_percent,
            dev.mute
        ))
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn get_default_source() -> Result<CallToolResult, McpError> {
    let result = tokio::task::spawn_blocking(|| {
        let mut handler = SourceController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        let dev = handler
            .get_default_device()
            .map_err(|e| format!("Failed to get default source: {:?}", e))?;

        let vol_percent = dev
            .volume
            .get()
            .first()
            .map(|v| v.0 as f64 / 65536.0 * 100.0)
            .unwrap_or(0.0);
        Ok::<_, String>(format!(
            "Default source:\n  [{}] {} - {}\n  Volume: {:.0}%, Muted: {}",
            dev.index,
            dev.name.as_deref().unwrap_or("unknown"),
            dev.description.as_deref().unwrap_or(""),
            vol_percent,
            dev.mute
        ))
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn set_default_sink(params: NameParams) -> Result<CallToolResult, McpError> {
    let name = params.name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut handler = SinkController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        handler
            .set_default_device(&name)
            .map_err(|e| format!("Failed to set default sink: {:?}", e))?;
        Ok::<_, String>(format!("Default sink set to: {}", name))
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn set_default_source(params: NameParams) -> Result<CallToolResult, McpError> {
    let name = params.name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut handler = SourceController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        handler
            .set_default_device(&name)
            .map_err(|e| format!("Failed to set default source: {:?}", e))?;
        Ok::<_, String>(format!("Default source set to: {}", name))
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn set_sink_input_volume(params: VolumeParams) -> Result<CallToolResult, McpError> {
    let index = params.index;
    let delta = params.delta;
    let result = tokio::task::spawn_blocking(move || {
        let mut handler = SinkController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;

        if delta >= 0.0 {
            handler.increase_app_volume_by_percent(index, delta);
        } else {
            handler.decrease_app_volume_by_percent(index, delta.abs());
        }

        Ok::<_, String>(format!("Volume for app {} adjusted by {:.0}%", index, delta))
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn set_sink_input_mute(params: MuteParams) -> Result<CallToolResult, McpError> {
    let index = params.index;
    let mute = params.mute;
    let result = tokio::task::spawn_blocking(move || {
        let mut handler = SinkController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        handler
            .set_app_mute(index, mute)
            .map_err(|e| format!("Failed to set mute: {:?}", e))?;
        Ok::<_, String>(format!(
            "App {} {}",
            index,
            if mute { "muted" } else { "unmuted" }
        ))
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn move_sink_input(params: MoveAppParams) -> Result<CallToolResult, McpError> {
    let app_index = params.app_index;
    let device_name = params.device_name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut handler = SinkController::create()
            .map_err(|e| format!("Failed to connect to PulseAudio: {:?}", e))?;
        handler
            .move_app_by_name(app_index, &device_name)
            .map_err(|e| format!("Failed to move app: {:?}", e))?;
        Ok::<_, String>(format!("App {} moved to sink '{}'", app_index, device_name))
    })
    .await
    .map_err(|e| internal_error(format!("Task failed: {}", e)))?;

    match result {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}
