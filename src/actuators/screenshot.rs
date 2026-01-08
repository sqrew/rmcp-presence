//! Screenshot actuators

use base64::{engine::general_purpose::STANDARD, Engine};
use image::ImageFormat;
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use std::io::Cursor;
use xcap::{Monitor, Window};

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CaptureMonitorParams {
    #[schemars(description = "Monitor index (0-based). Use list_monitors to see available monitors. Defaults to primary monitor if not specified.")]
    pub monitor_index: Option<usize>,
    #[schemars(description = "Image quality/size: 'full' (original), 'half' (50%), 'quarter' (25%). Defaults to 'quarter'.")]
    pub quality: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CaptureWindowParams {
    #[schemars(description = "Window title to capture (partial match, case-insensitive)")]
    pub title: String,
    #[schemars(description = "Image quality/size: 'full' (original), 'half' (50%), 'quarter' (25%). Defaults to 'quarter'.")]
    pub quality: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CaptureRegionParams {
    #[schemars(description = "X coordinate of top-left corner")]
    pub x: i32,
    #[schemars(description = "Y coordinate of top-left corner")]
    pub y: i32,
    #[schemars(description = "Width of region to capture")]
    pub width: u32,
    #[schemars(description = "Height of region to capture")]
    pub height: u32,
    #[schemars(description = "Monitor index (0-based). Defaults to primary monitor.")]
    pub monitor_index: Option<usize>,
}

// === Helper Functions ===

fn encode_image(img: image::RgbaImage, quality: Option<&str>) -> Result<String, String> {
    use image::imageops::FilterType;

    let scale = match quality.unwrap_or("quarter") {
        "full" => 1.0,
        "half" => 0.5,
        "quarter" => 0.25,
        _ => 0.25,
    };

    let img = if scale < 1.0 {
        let new_width = (img.width() as f64 * scale) as u32;
        let new_height = (img.height() as f64 * scale) as u32;
        image::imageops::resize(&img, new_width, new_height, FilterType::Triangle)
    } else {
        img
    };

    let mut buf = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;

    Ok(STANDARD.encode(buf.into_inner()))
}

// === Tool Functions ===

pub async fn list_monitors() -> Result<CallToolResult, McpError> {
    match Monitor::all() {
        Ok(monitors) => {
            let list: Vec<String> = monitors
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    format!(
                        "{}: {} ({}x{}){}",
                        i,
                        m.name(),
                        m.width(),
                        m.height(),
                        if m.is_primary() { " [primary]" } else { "" }
                    )
                })
                .collect();

            if list.is_empty() {
                Ok(CallToolResult::success(vec![Content::text(
                    "No monitors found",
                )]))
            } else {
                Ok(CallToolResult::success(vec![Content::text(list.join("\n"))]))
            }
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to list monitors: {}",
            e
        ))])),
    }
}

pub async fn capture_monitor(params: CaptureMonitorParams) -> Result<CallToolResult, McpError> {
    let monitors = match Monitor::all() {
        Ok(m) => m,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to get monitors: {}",
                e
            ))]));
        }
    };

    if monitors.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No monitors found",
        )]));
    }

    let monitor = if let Some(idx) = params.monitor_index {
        monitors.get(idx).ok_or_else(|| {
            format!(
                "Monitor index {} out of range (0-{})",
                idx,
                monitors.len() - 1
            )
        })
    } else {
        monitors
            .iter()
            .find(|m| m.is_primary())
            .or(monitors.first())
            .ok_or_else(|| "No monitors available".to_string())
    };

    let monitor = match monitor {
        Ok(m) => m,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(e)]));
        }
    };

    match monitor.capture_image() {
        Ok(img) => match encode_image(img, params.quality.as_deref()) {
            Ok(base64) => Ok(CallToolResult::success(vec![Content::image(
                base64,
                "image/png",
            )])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
        },
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to capture monitor: {}",
            e
        ))])),
    }
}

pub async fn list_windows() -> Result<CallToolResult, McpError> {
    match Window::all() {
        Ok(windows) => {
            let list: Vec<String> = windows
                .iter()
                .enumerate()
                .filter(|(_, w)| !w.is_minimized())
                .map(|(i, w)| {
                    format!(
                        "{}: \"{}\" ({}x{} at {},{})",
                        i,
                        w.title(),
                        w.width(),
                        w.height(),
                        w.x(),
                        w.y()
                    )
                })
                .collect();

            if list.is_empty() {
                Ok(CallToolResult::success(vec![Content::text(
                    "No visible windows found",
                )]))
            } else {
                Ok(CallToolResult::success(vec![Content::text(list.join("\n"))]))
            }
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to list windows: {}",
            e
        ))])),
    }
}

pub async fn capture_window(params: CaptureWindowParams) -> Result<CallToolResult, McpError> {
    let windows = match Window::all() {
        Ok(w) => w,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to get windows: {}",
                e
            ))]));
        }
    };

    let search = params.title.to_lowercase();
    let window = windows
        .iter()
        .find(|w| w.title().to_lowercase().contains(&search));

    let window = match window {
        Some(w) => w,
        None => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No window found matching '{}'",
                params.title
            ))]));
        }
    };

    match window.capture_image() {
        Ok(img) => match encode_image(img, params.quality.as_deref()) {
            Ok(base64) => Ok(CallToolResult::success(vec![Content::image(
                base64,
                "image/png",
            )])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
        },
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to capture window '{}': {}",
            window.title(),
            e
        ))])),
    }
}

pub async fn capture_region(params: CaptureRegionParams) -> Result<CallToolResult, McpError> {
    let monitors = match Monitor::all() {
        Ok(m) => m,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to get monitors: {}",
                e
            ))]));
        }
    };

    if monitors.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No monitors found",
        )]));
    }

    let monitor = if let Some(idx) = params.monitor_index {
        monitors.get(idx)
    } else {
        monitors
            .iter()
            .find(|m| m.is_primary())
            .or(monitors.first())
    };

    let monitor = match monitor {
        Some(m) => m,
        None => {
            return Ok(CallToolResult::success(vec![Content::text(
                "No monitor found",
            )]));
        }
    };

    match monitor.capture_image() {
        Ok(full_img) => {
            let x = params.x.max(0) as u32;
            let y = params.y.max(0) as u32;
            let width = params.width.min(full_img.width().saturating_sub(x));
            let height = params.height.min(full_img.height().saturating_sub(y));

            if width == 0 || height == 0 {
                return Ok(CallToolResult::success(vec![Content::text(
                    "Region is out of bounds or has zero size",
                )]));
            }

            let cropped = image::imageops::crop_imm(&full_img, x, y, width, height).to_image();

            match encode_image(cropped, Some("full")) {
                Ok(base64) => Ok(CallToolResult::success(vec![Content::image(
                    base64,
                    "image/png",
                )])),
                Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
            }
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to capture region: {}",
            e
        ))])),
    }
}
