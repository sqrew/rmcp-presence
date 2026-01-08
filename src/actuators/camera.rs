//! Camera capture tools

use crate::shared::internal_error;
use base64::Engine;
use nokhwa::{
    pixel_format::RgbFormat,
    query,
    utils::{RequestedFormat, RequestedFormatType},
    Camera,
};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for capturing from camera
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CaptureParams {
    /// Camera index (0-based). Defaults to 0 (first camera).
    #[serde(default)]
    pub index: Option<u32>,
    /// Image quality: 'full', 'half', 'quarter'. Defaults to 'quarter'.
    #[serde(default)]
    pub quality: Option<String>,
}

/// Parameters for camera index
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CameraIndexParams {
    /// Camera index (0-based). Defaults to 0.
    #[serde(default)]
    pub index: Option<u32>,
}

pub async fn list_cameras() -> Result<CallToolResult, McpError> {
    let cameras = query(nokhwa::utils::ApiBackend::Auto)
        .map_err(|e| internal_error(format!("Failed to query cameras: {}", e)))?;

    if cameras.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No cameras found.",
        )]));
    }

    let mut output = String::from("Available Cameras:\n\n");
    for (idx, cam) in cameras.iter().enumerate() {
        output.push_str(&format!(
            "Camera {}: {}\n  Index: {:?}\n  Description: {}\n\n",
            idx,
            cam.human_name(),
            cam.index(),
            cam.description(),
        ));
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn capture_camera(params: CaptureParams) -> Result<CallToolResult, McpError> {
    let index = params.index.unwrap_or(0);
    let quality = params.quality.unwrap_or_else(|| "quarter".to_string());

    let cameras = query(nokhwa::utils::ApiBackend::Auto)
        .map_err(|e| internal_error(format!("Failed to query cameras: {}", e)))?;

    if cameras.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "No cameras found.",
        )]));
    }

    let cam_idx = index as usize;
    if cam_idx >= cameras.len() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Camera index {} not found. Available: 0-{}",
            index,
            cameras.len() - 1
        ))]));
    }

    let camera_info = &cameras[cam_idx];
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

    let mut camera = Camera::new(camera_info.index().clone(), requested)
        .map_err(|e| internal_error(format!("Failed to open camera: {}", e)))?;

    camera.open_stream()
        .map_err(|e| internal_error(format!("Failed to open camera stream: {}", e)))?;

    let frame = camera.frame()
        .map_err(|e| internal_error(format!("Failed to capture frame: {}", e)))?;

    let decoded = frame.decode_image::<RgbFormat>()
        .map_err(|e| internal_error(format!("Failed to decode frame: {}", e)))?;

    let scale = match quality.as_str() {
        "full" => 1.0,
        "half" => 0.5,
        "quarter" => 0.25,
        _ => 0.25,
    };

    let img = image::DynamicImage::ImageRgb8(decoded);
    let img = if scale < 1.0 {
        let new_width = (img.width() as f64 * scale) as u32;
        let new_height = (img.height() as f64 * scale) as u32;
        img.resize(new_width, new_height, image::imageops::FilterType::Triangle)
    } else {
        img
    };

    let mut jpeg_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut jpeg_bytes);
    img.write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| internal_error(format!("Failed to encode JPEG: {}", e)))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes);

    Ok(CallToolResult::success(vec![Content::image(b64, "image/jpeg")]))
}

pub async fn get_camera_info(params: CameraIndexParams) -> Result<CallToolResult, McpError> {
    let index = params.index.unwrap_or(0);

    let cameras = query(nokhwa::utils::ApiBackend::Auto)
        .map_err(|e| internal_error(format!("Failed to query cameras: {}", e)))?;

    if cameras.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "No cameras found.",
        )]));
    }

    let cam_idx = index as usize;
    if cam_idx >= cameras.len() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Camera index {} not found. Available: 0-{}",
            index,
            cameras.len() - 1
        ))]));
    }

    let cam = &cameras[cam_idx];
    let output = format!(
        "Camera {}:\n  Name: {}\n  Index: {:?}\n  Description: {}\n  Misc: {}",
        index,
        cam.human_name(),
        cam.index(),
        cam.description(),
        cam.misc(),
    );

    Ok(CallToolResult::success(vec![Content::text(output)]))
}
