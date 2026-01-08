//! breakrs reminder integration

use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use std::process::Command;

use crate::shared::internal_error;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetReminderParams {
    #[schemars(
        description = "Duration and message in natural language (e.g. '5m get coffee', '1h meeting reminder', '30s tea is ready')"
    )]
    pub input: String,
    #[schemars(description = "Mark notification as urgent/critical")]
    #[serde(default)]
    pub urgent: bool,
    #[schemars(description = "Play sound with notification")]
    #[serde(default)]
    pub sound: bool,
    #[schemars(description = "Make timer recurring (repeats after completion)")]
    #[serde(default)]
    pub recurring: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveReminderParams {
    #[schemars(description = "Timer ID to remove (get IDs from list_reminders)")]
    pub id: String,
}

// === Tool Functions ===

pub async fn set_reminder(params: SetReminderParams) -> Result<CallToolResult, McpError> {
    let mut cmd = Command::new("breakrs");

    if params.urgent {
        cmd.arg("--urgent");
    }
    if params.sound {
        cmd.arg("--sound");
    }
    if params.recurring {
        cmd.arg("--recurring");
    }

    cmd.arg(&params.input);

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Reminder set: {}",
                    stdout.trim()
                ))]))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Failed to set reminder: {}",
                    stderr.trim()
                ))]))
            }
        }
        Err(e) => Err(internal_error(format!("Failed to run breakrs: {}", e))),
    }
}

pub async fn list_reminders() -> Result<CallToolResult, McpError> {
    match Command::new("breakrs").arg("list").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                Ok(CallToolResult::success(vec![Content::text(
                    "No active reminders",
                )]))
            } else {
                Ok(CallToolResult::success(vec![Content::text(
                    stdout.to_string(),
                )]))
            }
        }
        Err(e) => Err(internal_error(format!("Failed to run breakrs: {}", e))),
    }
}

pub async fn remove_reminder(params: RemoveReminderParams) -> Result<CallToolResult, McpError> {
    match Command::new("breakrs")
        .arg("remove")
        .arg(&params.id)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Removed reminder {}",
                    params.id
                ))]))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Failed to remove reminder: {}",
                    stderr.trim()
                ))]))
            }
        }
        Err(e) => Err(internal_error(format!("Failed to run breakrs: {}", e))),
    }
}

pub async fn clear_reminders() -> Result<CallToolResult, McpError> {
    match Command::new("breakrs").arg("clear").output() {
        Ok(output) => {
            if output.status.success() {
                Ok(CallToolResult::success(vec![Content::text(
                    "Cleared all reminders",
                )]))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Failed to clear reminders: {}",
                    stderr.trim()
                ))]))
            }
        }
        Err(e) => Err(internal_error(format!("Failed to run breakrs: {}", e))),
    }
}

pub async fn daemon_status() -> Result<CallToolResult, McpError> {
    match Command::new("breakrs").arg("status").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(CallToolResult::success(vec![Content::text(
                stdout.to_string(),
            )]))
        }
        Err(e) => Err(internal_error(format!("Failed to run breakrs: {}", e))),
    }
}

pub async fn get_history() -> Result<CallToolResult, McpError> {
    match Command::new("breakrs").arg("history").output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.trim().is_empty() {
                Ok(CallToolResult::success(vec![Content::text(
                    "No reminder history",
                )]))
            } else {
                Ok(CallToolResult::success(vec![Content::text(
                    stdout.to_string(),
                )]))
            }
        }
        Err(e) => Err(internal_error(format!("Failed to run breakrs: {}", e))),
    }
}
