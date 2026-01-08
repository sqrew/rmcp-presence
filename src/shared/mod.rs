//! Shared utilities for rmcp-presence

use rmcp::ErrorData as McpError;

/// Format a duration in seconds to human readable string
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        if secs == 0 {
            format!("{}m", mins)
        } else {
            format!("{}m {}s", mins, secs)
        }
    } else {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        if mins == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, mins)
        }
    }
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Create an internal error
pub fn internal_error(msg: impl Into<String>) -> McpError {
    McpError::internal_error(msg.into(), None)
}
