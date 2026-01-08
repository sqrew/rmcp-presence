//! Mouse and keyboard automation via xdotool

use crate::shared::internal_error;
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use std::process::Command;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveMouseParams {
    #[schemars(description = "X coordinate")]
    pub x: i32,
    #[schemars(description = "Y coordinate")]
    pub y: i32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClickParams {
    #[schemars(description = "Button to click: 1 (left), 2 (middle), 3 (right). Default: 1")]
    #[serde(default = "default_button")]
    pub button: u8,
}

fn default_button() -> u8 {
    1
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClickAtParams {
    #[schemars(description = "X coordinate")]
    pub x: i32,
    #[schemars(description = "Y coordinate")]
    pub y: i32,
    #[schemars(description = "Button to click: 1 (left), 2 (middle), 3 (right). Default: 1")]
    #[serde(default = "default_button")]
    pub button: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TypeTextParams {
    #[schemars(description = "Text to type")]
    pub text: String,
    #[schemars(description = "Delay between keystrokes in milliseconds. Default: 12")]
    #[serde(default = "default_delay")]
    pub delay: u32,
}

fn default_delay() -> u32 {
    12
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KeyPressParams {
    #[schemars(description = "Key(s) to press. Examples: Return, Escape, ctrl+c, alt+Tab, super+1")]
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScrollParams {
    #[schemars(description = "Scroll direction: up, down, left, right")]
    pub direction: String,
    #[schemars(description = "Number of clicks to scroll. Default: 3")]
    #[serde(default = "default_clicks")]
    pub clicks: u32,
}

fn default_clicks() -> u32 {
    3
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchWindowParams {
    #[schemars(description = "Search query (window name, class, or pattern)")]
    pub query: String,
    #[schemars(description = "Search by: 'name', 'class', 'classname', or 'any' (default: 'any')")]
    #[serde(default = "default_search_type")]
    pub search_type: String,
}

fn default_search_type() -> String {
    "any".to_string()
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WindowIdParams {
    #[schemars(description = "Window ID (from search_window or get_active_window)")]
    pub window_id: String,
}

// === Helper Functions ===

fn button_name(button: u8) -> &'static str {
    match button {
        1 => "left",
        2 => "middle",
        3 => "right",
        _ => "unknown",
    }
}

// === Tool Functions ===

pub async fn move_mouse(params: MoveMouseParams) -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args(["mousemove", &params.x.to_string(), &params.y.to_string()])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Mouse moved to ({}, {})",
            params.x, params.y
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn click(params: ClickParams) -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args(["click", &params.button.to_string()])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Clicked {} mouse button",
            button_name(params.button)
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn click_at(params: ClickAtParams) -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args([
            "mousemove",
            &params.x.to_string(),
            &params.y.to_string(),
            "click",
            &params.button.to_string(),
        ])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Clicked {} at ({}, {})",
            button_name(params.button),
            params.x,
            params.y
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn type_text(params: TypeTextParams) -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args(["type", "--delay", &params.delay.to_string(), &params.text])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Typed: \"{}\"",
            params.text
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn key_press(params: KeyPressParams) -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args(["key", &params.key])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Pressed key: {}",
            params.key
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn scroll(params: ScrollParams) -> Result<CallToolResult, McpError> {
    let button = match params.direction.to_lowercase().as_str() {
        "up" => "4",
        "down" => "5",
        "left" => "6",
        "right" => "7",
        _ => {
            return Err(internal_error(
                "Invalid direction. Use: up, down, left, right",
            ))
        }
    };

    let output = Command::new("xdotool")
        .args(["click", "--repeat", &params.clicks.to_string(), button])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Scrolled {} {} clicks",
            params.direction, params.clicks
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn get_mouse_position() -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args(["getmouselocation", "--shell"])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut x = 0;
        let mut y = 0;
        for line in stdout.lines() {
            if line.starts_with("X=") {
                x = line[2..].parse().unwrap_or(0);
            } else if line.starts_with("Y=") {
                y = line[2..].parse().unwrap_or(0);
            }
        }
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Mouse position: ({}, {})",
            x, y
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn double_click() -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args(["click", "--repeat", "2", "1"])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        Ok(CallToolResult::success(vec![Content::text(
            "Double-clicked".to_string(),
        )]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn search_window(params: SearchWindowParams) -> Result<CallToolResult, McpError> {
    let mut args = vec!["search"];

    match params.search_type.to_lowercase().as_str() {
        "name" => args.push("--name"),
        "class" => args.push("--class"),
        "classname" => args.push("--classname"),
        _ => {} // 'any' uses default behavior
    }

    args.push(&params.query);

    let output = Command::new("xdotool")
        .args(&args)
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let window_ids: Vec<&str> = stdout.lines().collect();

        if window_ids.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "No windows found matching '{}'",
                params.query
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Found {} window(s):\n{}",
                window_ids.len(),
                stdout.trim()
            ))]))
        }
    } else {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "No windows found matching '{}'",
            params.query
        ))]))
    }
}

pub async fn get_active_window() -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args(["getactivewindow"])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        let window_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Active window ID: {}",
            window_id
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn get_window_geometry(params: WindowIdParams) -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args(["getwindowgeometry", "--shell", &params.window_id])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut x = 0;
        let mut y = 0;
        let mut width = 0;
        let mut height = 0;
        let mut screen = 0;

        for line in stdout.lines() {
            if line.starts_with("X=") {
                x = line[2..].parse().unwrap_or(0);
            } else if line.starts_with("Y=") {
                y = line[2..].parse().unwrap_or(0);
            } else if line.starts_with("WIDTH=") {
                width = line[6..].parse().unwrap_or(0);
            } else if line.starts_with("HEIGHT=") {
                height = line[7..].parse().unwrap_or(0);
            } else if line.starts_with("SCREEN=") {
                screen = line[7..].parse().unwrap_or(0);
            }
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Window {} geometry:\n  Position: ({}, {})\n  Size: {}x{}\n  Screen: {}",
            params.window_id, x, y, width, height, screen
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

pub async fn get_window_name(params: WindowIdParams) -> Result<CallToolResult, McpError> {
    let output = Command::new("xdotool")
        .args(["getwindowname", &params.window_id])
        .output()
        .map_err(|e| internal_error(format!("Failed to run xdotool: {}", e)))?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Window {} title: {}",
            params.window_id, name
        ))]))
    } else {
        Err(internal_error(format!(
            "xdotool error: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}
