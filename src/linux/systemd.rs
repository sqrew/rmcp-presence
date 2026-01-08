//! systemd service management

use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use std::process::Command;
use zbus::Connection;
use zbus_systemd::systemd1::ManagerProxy;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnitParams {
    #[schemars(description = "Unit name (e.g., \"nginx.service\", \"docker.service\")")]
    pub unit: String,
    #[schemars(description = "Use user session instead of system (default: false)")]
    #[serde(default)]
    pub user: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListParams {
    #[schemars(description = "Filter by unit type (e.g., \"service\", \"timer\", \"socket\")")]
    pub unit_type: Option<String>,
    #[schemars(description = "Filter by active state (e.g., \"active\", \"inactive\", \"failed\")")]
    pub state: Option<String>,
    #[schemars(description = "Use user session instead of system (default: false)")]
    #[serde(default)]
    pub user: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LogsParams {
    #[schemars(description = "Unit name (e.g., \"nginx.service\")")]
    pub unit: String,
    #[schemars(description = "Number of recent log lines to fetch (default: 50)")]
    #[serde(default)]
    pub lines: Option<u32>,
    #[schemars(description = "Use user session instead of system (default: false)")]
    #[serde(default)]
    pub user: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FailedParams {
    #[schemars(description = "Use user session instead of system (default: false)")]
    #[serde(default)]
    pub user: Option<bool>,
}

// === Helper Functions ===

async fn get_connection(user: bool) -> Result<Connection, String> {
    if user {
        Connection::session()
            .await
            .map_err(|e| format!("Failed to connect to user session bus: {}", e))
    } else {
        Connection::system()
            .await
            .map_err(|e| format!("Failed to connect to system bus: {}", e))
    }
}

async fn get_manager(conn: &Connection) -> Result<ManagerProxy<'_>, String> {
    ManagerProxy::new(conn)
        .await
        .map_err(|e| format!("Failed to get systemd manager: {}", e))
}

fn chrono_lite(unix_secs: u64) -> String {
    let secs = unix_secs;
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{} days ago", days)
    } else if hours > 0 {
        format!("{} hours ago", hours % 24)
    } else if mins > 0 {
        format!("{} minutes ago", mins % 60)
    } else {
        format!("{} seconds ago", secs % 60)
    }
}

// === Tool Functions ===

pub async fn list_units(params: ListParams) -> Result<CallToolResult, McpError> {
    let user = params.user.unwrap_or(false);
    let conn = match get_connection(user).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let manager = match get_manager(&conn).await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.list_units().await {
        Ok(units) => {
            let filtered: Vec<String> = units
                .iter()
                .filter(|u| {
                    if let Some(ref ut) = params.unit_type {
                        let ext = format!(".{}", ut);
                        if !u.0.ends_with(&ext) {
                            return false;
                        }
                    }
                    if let Some(ref state) = params.state {
                        if u.3.to_lowercase() != state.to_lowercase() {
                            return false;
                        }
                    }
                    true
                })
                .map(|u| format!("{} ({}) - {}", u.0, u.3, u.1))
                .collect();

            if filtered.is_empty() {
                Ok(CallToolResult::success(vec![Content::text(
                    "No units found matching criteria",
                )]))
            } else {
                let scope = if user { "user" } else { "system" };
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "{} units ({}):\n{}",
                    filtered.len(),
                    scope,
                    filtered.join("\n")
                ))]))
            }
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to list units: {}",
            e
        ))])),
    }
}

pub async fn get_unit_status(params: UnitParams) -> Result<CallToolResult, McpError> {
    let user = params.user.unwrap_or(false);
    let conn = match get_connection(user).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let manager = match get_manager(&conn).await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let unit_path = match manager.get_unit(params.unit.clone()).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to get unit '{}': {}",
                params.unit, e
            ))]))
        }
    };

    let unit_proxy = match zbus_systemd::systemd1::UnitProxy::builder(&conn)
        .path(unit_path)
        .ok()
    {
        Some(builder) => match builder.build().await {
            Ok(p) => p,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Failed to connect to unit: {}",
                    e
                ))]))
            }
        },
        None => {
            return Ok(CallToolResult::success(vec![Content::text(
                "Invalid unit path",
            )]))
        }
    };

    let mut info = Vec::new();
    let scope = if user { "user" } else { "system" };
    info.push(format!("Unit: {} ({})", params.unit, scope));

    if let Ok(desc) = unit_proxy.description().await {
        info.push(format!("Description: {}", desc));
    }

    if let Ok(load) = unit_proxy.load_state().await {
        info.push(format!("Load State: {}", load));
    }

    if let Ok(active) = unit_proxy.active_state().await {
        info.push(format!("Active State: {}", active));
    }

    if let Ok(sub) = unit_proxy.sub_state().await {
        info.push(format!("Sub State: {}", sub));
    }

    if let Ok(since) = unit_proxy.active_enter_timestamp().await {
        if since > 0 {
            let secs = since / 1_000_000;
            let datetime = chrono_lite(secs);
            info.push(format!("Active Since: {}", datetime));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(info.join("\n"))]))
}

pub async fn start_unit(params: UnitParams) -> Result<CallToolResult, McpError> {
    let user = params.user.unwrap_or(false);
    let conn = match get_connection(user).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let manager = match get_manager(&conn).await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager
        .start_unit(params.unit.clone(), "replace".to_string())
        .await
    {
        Ok(_job) => {
            let scope = if user { "user" } else { "system" };
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Started {} ({})",
                params.unit, scope
            ))]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to start '{}': {}",
            params.unit, e
        ))])),
    }
}

pub async fn stop_unit(params: UnitParams) -> Result<CallToolResult, McpError> {
    let user = params.user.unwrap_or(false);
    let conn = match get_connection(user).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let manager = match get_manager(&conn).await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager
        .stop_unit(params.unit.clone(), "replace".to_string())
        .await
    {
        Ok(_job) => {
            let scope = if user { "user" } else { "system" };
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Stopped {} ({})",
                params.unit, scope
            ))]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to stop '{}': {}",
            params.unit, e
        ))])),
    }
}

pub async fn restart_unit(params: UnitParams) -> Result<CallToolResult, McpError> {
    let user = params.user.unwrap_or(false);
    let conn = match get_connection(user).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let manager = match get_manager(&conn).await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager
        .restart_unit(params.unit.clone(), "replace".to_string())
        .await
    {
        Ok(_job) => {
            let scope = if user { "user" } else { "system" };
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Restarted {} ({})",
                params.unit, scope
            ))]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to restart '{}': {}",
            params.unit, e
        ))])),
    }
}

pub async fn list_failed_units(params: FailedParams) -> Result<CallToolResult, McpError> {
    let user = params.user.unwrap_or(false);
    let conn = match get_connection(user).await {
        Ok(c) => c,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let manager = match get_manager(&conn).await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.list_units().await {
        Ok(units) => {
            let failed: Vec<String> = units
                .iter()
                .filter(|u| u.3.to_lowercase() == "failed")
                .map(|u| format!("{} - {}", u.0, u.1))
                .collect();

            let scope = if user { "user" } else { "system" };
            if failed.is_empty() {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "No failed units ({})",
                    scope
                ))]))
            } else {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "{} failed units ({}):\n{}",
                    failed.len(),
                    scope,
                    failed.join("\n")
                ))]))
            }
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to list units: {}",
            e
        ))])),
    }
}

pub async fn get_unit_logs(params: LogsParams) -> Result<CallToolResult, McpError> {
    let lines = params.lines.unwrap_or(50);
    let user = params.user.unwrap_or(false);

    let mut cmd = Command::new("journalctl");
    cmd.arg("-u").arg(&params.unit);
    cmd.arg("-n").arg(lines.to_string());
    cmd.arg("--no-pager");

    if user {
        cmd.arg("--user");
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let logs = String::from_utf8_lossy(&output.stdout);
                if logs.trim().is_empty() {
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "No logs found for {}",
                        params.unit
                    ))]))
                } else {
                    Ok(CallToolResult::success(vec![Content::text(
                        logs.to_string(),
                    )]))
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "journalctl error: {}",
                    stderr
                ))]))
            }
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to run journalctl: {}",
            e
        ))])),
    }
}
