//! File/URL opening actuators

use crate::shared::internal_error;
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenPathParams {
    #[schemars(description = "Path to file/folder or URL to open")]
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenWithParams {
    #[schemars(description = "Path to file/folder or URL to open")]
    pub path: String,
    #[schemars(description = "Application to open with (e.g. 'firefox', 'code', 'vlc')")]
    pub app: String,
}

// === Tool Functions ===

pub async fn open_path(params: OpenPathParams) -> Result<CallToolResult, McpError> {
    open::that(&params.path)
        .map_err(|e| internal_error(format!("Failed to open '{}': {}", params.path, e)))?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Opened: {}",
        params.path
    ))]))
}

pub async fn open_with(params: OpenWithParams) -> Result<CallToolResult, McpError> {
    open::with(&params.path, &params.app)
        .map_err(|e| internal_error(format!("Failed to open '{}' with '{}': {}", params.path, params.app, e)))?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Opened {} with {}",
        params.path, params.app
    ))]))
}
