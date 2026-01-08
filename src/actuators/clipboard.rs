//! System clipboard actuators

use crate::shared::internal_error;
use arboard::Clipboard;
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteClipboardParams {
    #[schemars(description = "Text to write to the clipboard")]
    pub text: String,
}

// === Tool Functions ===

pub async fn read_clipboard() -> Result<CallToolResult, McpError> {
    let mut clipboard =
        Clipboard::new().map_err(|e| internal_error(format!("Failed to access clipboard: {}", e)))?;

    match clipboard.get_text() {
        Ok(text) => {
            if text.is_empty() {
                Ok(CallToolResult::success(vec![Content::text(
                    "Clipboard is empty".to_string(),
                )]))
            } else {
                let preview = if text.len() > 500 {
                    format!("{}... ({} chars total)", &text[..500], text.len())
                } else {
                    text.clone()
                };
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Clipboard contents:\n{}",
                    preview
                ))]))
            }
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Could not read clipboard (may contain non-text data): {}",
            e
        ))])),
    }
}

pub async fn write_clipboard(params: WriteClipboardParams) -> Result<CallToolResult, McpError> {
    let mut clipboard =
        Clipboard::new().map_err(|e| internal_error(format!("Failed to access clipboard: {}", e)))?;

    clipboard
        .set_text(&params.text)
        .map_err(|e| internal_error(format!("Failed to write to clipboard: {}", e)))?;

    let preview = if params.text.len() > 100 {
        format!("{}... ({} chars)", &params.text[..100], params.text.len())
    } else {
        params.text.clone()
    };

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Copied to clipboard: {}",
        preview
    ))]))
}

pub async fn clear_clipboard() -> Result<CallToolResult, McpError> {
    let mut clipboard =
        Clipboard::new().map_err(|e| internal_error(format!("Failed to access clipboard: {}", e)))?;

    clipboard
        .clear()
        .map_err(|e| internal_error(format!("Failed to clear clipboard: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(
        "Clipboard cleared".to_string(),
    )]))
}
