//! Audio status - comprehensive audio state in one call

use crate::shared::internal_error;
use pulsectl::controllers::{AppControl, DeviceControl, SinkController, SourceController};
use rmcp::{model::{CallToolResult, Content}, ErrorData as McpError};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AudioStatus {
    // Basic state
    pub volume_percent: u8,
    pub muted: bool,

    // Default devices
    pub default_output: Option<DeviceInfo>,
    pub default_input: Option<DeviceInfo>,

    // What's playing
    pub now_playing: Option<NowPlaying>,

    // Apps using audio
    pub apps_playing: Vec<AppAudio>,
    pub apps_recording: Vec<AppAudio>,
}

#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub name: String,
    pub description: String,
    pub volume_percent: u32,
    pub muted: bool,
}

#[derive(Debug, Serialize)]
pub struct NowPlaying {
    pub player: String,
    pub status: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AppAudio {
    pub name: String,
    pub index: u32,
    pub volume_percent: u32,
    pub muted: bool,
}

pub async fn get_audio_status() -> Result<CallToolResult, McpError> {
    let status = tokio::task::spawn_blocking(build_audio_status)
        .await
        .map_err(|e| internal_error(format!("Task error: {}", e)))?
        .map_err(|e| internal_error(e))?;

    let json = serde_json::to_string_pretty(&status)
        .map_err(|e| internal_error(format!("Serialization error: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

fn build_audio_status() -> Result<AudioStatus, String> {
    // Get basic volume/mute via cpvc (cross-platform)
    let volume_percent = cpvc::get_system_volume();
    let muted = cpvc::get_mute();

    // Get default output device
    let default_output = get_default_sink();

    // Get default input device
    let default_input = get_default_source();

    // Get apps playing audio
    let apps_playing = get_sink_inputs();

    // Get apps recording audio
    let apps_recording = get_source_outputs();

    // Get now playing via mpris
    let now_playing = get_now_playing();

    Ok(AudioStatus {
        volume_percent,
        muted,
        default_output,
        default_input,
        now_playing,
        apps_playing,
        apps_recording,
    })
}

fn get_default_sink() -> Option<DeviceInfo> {
    let mut handler = SinkController::create().ok()?;
    let default = handler.get_default_device().ok()?;

    let vol_percent = default
        .volume
        .get()
        .first()
        .map(|v| (v.0 as f64 / 65536.0 * 100.0) as u32)
        .unwrap_or(0);

    Some(DeviceInfo {
        name: default.name.unwrap_or_default(),
        description: default.description.unwrap_or_default(),
        volume_percent: vol_percent,
        muted: default.mute,
    })
}

fn get_default_source() -> Option<DeviceInfo> {
    let mut handler = SourceController::create().ok()?;
    let default = handler.get_default_device().ok()?;

    let vol_percent = default
        .volume
        .get()
        .first()
        .map(|v| (v.0 as f64 / 65536.0 * 100.0) as u32)
        .unwrap_or(0);

    Some(DeviceInfo {
        name: default.name.unwrap_or_default(),
        description: default.description.unwrap_or_default(),
        volume_percent: vol_percent,
        muted: default.mute,
    })
}

fn get_sink_inputs() -> Vec<AppAudio> {
    let Ok(mut handler) = SinkController::create() else {
        return vec![];
    };

    let Ok(apps) = handler.list_applications() else {
        return vec![];
    };

    apps.into_iter()
        .map(|app| {
            let vol_percent = app
                .volume
                .get()
                .first()
                .map(|v| (v.0 as f64 / 65536.0 * 100.0) as u32)
                .unwrap_or(0);

            AppAudio {
                name: app.name.unwrap_or_else(|| "Unknown".to_string()),
                index: app.index,
                volume_percent: vol_percent,
                muted: app.mute,
            }
        })
        .collect()
}

fn get_source_outputs() -> Vec<AppAudio> {
    let Ok(mut handler) = SourceController::create() else {
        return vec![];
    };

    let Ok(apps) = handler.list_applications() else {
        return vec![];
    };

    apps.into_iter()
        .map(|app| {
            let vol_percent = app
                .volume
                .get()
                .first()
                .map(|v| (v.0 as f64 / 65536.0 * 100.0) as u32)
                .unwrap_or(0);

            AppAudio {
                name: app.name.unwrap_or_else(|| "Unknown".to_string()),
                index: app.index,
                volume_percent: vol_percent,
                muted: app.mute,
            }
        })
        .collect()
}

fn get_now_playing() -> Option<NowPlaying> {
    let player = mpris::PlayerFinder::new().ok()?.find_active().ok()?;

    let status = match player.get_playback_status().ok()? {
        mpris::PlaybackStatus::Playing => "playing",
        mpris::PlaybackStatus::Paused => "paused",
        mpris::PlaybackStatus::Stopped => "stopped",
    };

    let metadata = player.get_metadata().ok()?;

    Some(NowPlaying {
        player: player.identity().to_string(),
        status: status.to_string(),
        title: metadata.title().map(|s| s.to_string()),
        artist: metadata.artists().and_then(|a| a.first().map(|s| s.to_string())),
        album: metadata.album_name().map(|s| s.to_string()),
    })
}
