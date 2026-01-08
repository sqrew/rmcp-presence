//! Display/monitor information sensors

use crate::shared::internal_error;
use display_info::DisplayInfo;
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// === Parameter Types ===

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DisplayPointParams {
    #[schemars(description = "X coordinate on screen")]
    pub x: i32,
    #[schemars(description = "Y coordinate on screen")]
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DisplayNameParams {
    #[schemars(description = "Display name to search for")]
    pub name: String,
}

// === Helper Functions ===

fn format_single_display(d: &DisplayInfo) -> String {
    let mut result = String::new();
    let primary = if d.is_primary { " (primary)" } else { "" };
    result.push_str(&format!(
        "{}{}\n",
        if d.friendly_name.is_empty() {
            &d.name
        } else {
            &d.friendly_name
        },
        primary
    ));
    result.push_str(&format!("  Resolution: {}x{}\n", d.width, d.height));
    result.push_str(&format!("  Position: ({}, {})\n", d.x, d.y));

    if d.width_mm > 0 && d.height_mm > 0 {
        let diag_mm = ((d.width_mm.pow(2) + d.height_mm.pow(2)) as f32).sqrt();
        let diag_inches = diag_mm / 25.4;
        result.push_str(&format!(
            "  Physical: {}mm x {}mm (~{:.1}\")\n",
            d.width_mm, d.height_mm, diag_inches
        ));
    }

    if d.frequency > 0.0 {
        result.push_str(&format!("  Refresh: {:.0}Hz\n", d.frequency));
    }

    if d.scale_factor != 1.0 {
        result.push_str(&format!("  Scale: {:.0}%\n", d.scale_factor * 100.0));
    }

    if d.rotation != 0.0 {
        result.push_str(&format!("  Rotation: {}°\n", d.rotation as i32));
    }

    result
}

// === Tool Functions ===

pub async fn get_display_info() -> Result<CallToolResult, McpError> {
    let displays = DisplayInfo::all()
        .map_err(|e| internal_error(format!("Failed to get display info: {}", e)))?;

    let mut result = String::from("Display Information:\n\n");

    if displays.is_empty() {
        result.push_str("No displays detected.\n");
    } else {
        for (i, d) in displays.iter().enumerate() {
            let primary = if d.is_primary { " (primary)" } else { "" };
            result.push_str(&format!(
                "Display {}: {}{}\n",
                i + 1,
                if d.friendly_name.is_empty() {
                    &d.name
                } else {
                    &d.friendly_name
                },
                primary
            ));
            result.push_str(&format!("  Resolution: {}x{}\n", d.width, d.height));
            result.push_str(&format!("  Position: ({}, {})\n", d.x, d.y));

            if d.width_mm > 0 && d.height_mm > 0 {
                let diag_mm = ((d.width_mm.pow(2) + d.height_mm.pow(2)) as f32).sqrt();
                let diag_inches = diag_mm / 25.4;
                result.push_str(&format!(
                    "  Physical: {}mm x {}mm (~{:.1}\")\n",
                    d.width_mm, d.height_mm, diag_inches
                ));
            }

            if d.frequency > 0.0 {
                result.push_str(&format!("  Refresh: {:.0}Hz\n", d.frequency));
            }

            if d.scale_factor != 1.0 {
                result.push_str(&format!("  Scale: {:.0}%\n", d.scale_factor * 100.0));
            }

            result.push('\n');
        }
        result.push_str(&format!("Total displays: {}\n", displays.len()));
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_display_at_point(params: DisplayPointParams) -> Result<CallToolResult, McpError> {
    let display = DisplayInfo::from_point(params.x, params.y).map_err(|e| {
        internal_error(format!(
            "Failed to get display at ({}, {}): {}",
            params.x, params.y, e
        ))
    })?;

    let formatted = format_single_display(&display);
    let result = format!("Display at ({}, {}):\n{}", params.x, params.y, formatted);

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_display_by_name(params: DisplayNameParams) -> Result<CallToolResult, McpError> {
    let display = DisplayInfo::from_name(&params.name)
        .map_err(|e| internal_error(format!("Failed to get display '{}': {}", params.name, e)))?;

    let formatted = format_single_display(&display);

    Ok(CallToolResult::success(vec![Content::text(formatted)]))
}
