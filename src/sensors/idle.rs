//! User idle time sensors

use crate::shared::{format_duration, internal_error};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use user_idle::UserIdle;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IdleThresholdParams {
    #[schemars(description = "Threshold in seconds to check against (default: 300)")]
    pub threshold_seconds: u64,
}

// === Tool Functions ===

pub async fn get_idle_time() -> Result<CallToolResult, McpError> {
    let idle =
        UserIdle::get_time().map_err(|e| internal_error(format!("Failed to get idle time: {}", e)))?;

    let seconds = idle.as_seconds();
    let formatted = format_duration(seconds);

    let result = format!(
        "User Idle Time:\n\n  Raw: {} seconds\n  Formatted: {}\n",
        seconds, formatted
    );

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn is_idle_for(params: IdleThresholdParams) -> Result<CallToolResult, McpError> {
    let idle =
        UserIdle::get_time().map_err(|e| internal_error(format!("Failed to get idle time: {}", e)))?;

    let seconds = idle.as_seconds();
    let threshold = params.threshold_seconds;
    let is_idle = seconds >= threshold;

    let result = format!(
        "Idle Check:\n\n  Current idle: {} ({})\n  Threshold: {} ({})\n  Is idle: {}\n",
        seconds,
        format_duration(seconds),
        threshold,
        format_duration(threshold),
        if is_idle { "YES" } else { "NO" }
    );

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
