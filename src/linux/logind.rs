//! systemd-logind power management

use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use zbus::Connection;
use zbus_systemd::login1::ManagerProxy;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InteractiveParams {
    #[schemars(description = "Show polkit authentication dialog if needed (default: false)")]
    #[serde(default)]
    pub interactive: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SessionIdParams {
    #[schemars(description = "Session ID to operate on (e.g., \"1\", \"c1\")")]
    pub session_id: String,
}

// === Helper Functions ===

async fn get_manager() -> Result<ManagerProxy<'static>, String> {
    let connection = Connection::system()
        .await
        .map_err(|e| format!("Failed to connect to system bus: {}", e))?;
    ManagerProxy::new(&connection)
        .await
        .map_err(|e| format!("Failed to create logind proxy: {}", e))
}

// === Tool Functions ===

pub async fn suspend(params: InteractiveParams) -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.suspend(params.interactive).await {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(
            "System suspended successfully",
        )])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to suspend: {}",
            e
        ))])),
    }
}

pub async fn hibernate(params: InteractiveParams) -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.hibernate(params.interactive).await {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(
            "System hibernated successfully",
        )])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to hibernate: {}",
            e
        ))])),
    }
}

pub async fn poweroff(params: InteractiveParams) -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.power_off(params.interactive).await {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(
            "System powering off...",
        )])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to power off: {}",
            e
        ))])),
    }
}

pub async fn reboot(params: InteractiveParams) -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.reboot(params.interactive).await {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(
            "System rebooting...",
        )])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to reboot: {}",
            e
        ))])),
    }
}

pub async fn lock_session(params: SessionIdParams) -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.lock_session(params.session_id.clone()).await {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Session '{}' locked",
            params.session_id
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to lock session '{}': {}",
            params.session_id, e
        ))])),
    }
}

pub async fn list_sessions() -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.list_sessions().await {
        Ok(sessions) => {
            if sessions.is_empty() {
                return Ok(CallToolResult::success(vec![Content::text(
                    "No active sessions",
                )]));
            }

            let mut output = format!("{} active session(s):\n\n", sessions.len());
            for (session_id, uid, user, seat, path) in sessions {
                output.push_str(&format!(
                    "  {} - user: {} (uid: {}), seat: {}\n",
                    session_id,
                    user,
                    uid,
                    if seat.is_empty() { "none" } else { &seat }
                ));
                let _ = path;
            }
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to list sessions: {}",
            e
        ))])),
    }
}

pub async fn can_suspend() -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.can_suspend().await {
        Ok(result) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Can suspend: {}",
            result
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to check: {}",
            e
        ))])),
    }
}

pub async fn can_hibernate() -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.can_hibernate().await {
        Ok(result) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Can hibernate: {}",
            result
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to check: {}",
            e
        ))])),
    }
}

pub async fn can_poweroff() -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.can_power_off().await {
        Ok(result) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Can power off: {}",
            result
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to check: {}",
            e
        ))])),
    }
}

pub async fn can_reboot() -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match manager.can_reboot().await {
        Ok(result) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Can reboot: {}",
            result
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to check: {}",
            e
        ))])),
    }
}

pub async fn get_capabilities() -> Result<CallToolResult, McpError> {
    let manager = match get_manager().await {
        Ok(m) => m,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    let mut output = String::from("Power capabilities:\n\n");

    if let Ok(result) = manager.can_suspend().await {
        output.push_str(&format!("  Suspend: {}\n", result));
    }
    if let Ok(result) = manager.can_hibernate().await {
        output.push_str(&format!("  Hibernate: {}\n", result));
    }
    if let Ok(result) = manager.can_power_off().await {
        output.push_str(&format!("  Power off: {}\n", result));
    }
    if let Ok(result) = manager.can_reboot().await {
        output.push_str(&format!("  Reboot: {}\n", result));
    }

    output.push_str("\nValues: 'yes' = allowed, 'challenge' = needs auth, 'no' = not available");

    Ok(CallToolResult::success(vec![Content::text(output)]))
}
