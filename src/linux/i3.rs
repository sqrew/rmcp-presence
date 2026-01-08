//! i3 window manager control

use crate::shared::internal_error;
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio_i3ipc::{reply::Node, I3};

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SwitchWorkspaceParams {
    #[schemars(description = "Workspace to switch to (number or name)")]
    pub workspace: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FocusWindowParams {
    #[schemars(description = "i3 criteria to match window, e.g. [class=\"Firefox\"] or [title=\"vim\"]")]
    pub criteria: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveToWorkspaceParams {
    #[schemars(description = "Workspace to move the focused window to")]
    pub workspace: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunCommandParams {
    #[schemars(description = "i3 command to execute (e.g. 'split h', 'layout tabbed', 'kill')")]
    pub command: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExecParams {
    #[schemars(description = "Command to execute, e.g. 'firefox', 'kitty', 'emacs'")]
    pub command: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KillWindowParams {
    #[schemars(
        description = "i3 criteria to match window to kill, e.g. [class=\"Firefox\"] or [title=\"~\"]"
    )]
    pub criteria: String,
}

// === Helper Functions ===

async fn connect() -> Result<I3, McpError> {
    I3::connect()
        .await
        .map_err(|e| internal_error(format!("Failed to connect to i3: {}", e)))
}

// === Tool Functions ===

pub async fn get_workspaces() -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let workspaces = conn
        .get_workspaces()
        .await
        .map_err(|e| internal_error(format!("Failed to get workspaces: {}", e)))?;

    let json = serde_json::to_string_pretty(&workspaces)
        .map_err(|e| internal_error(format!("Failed to serialize workspaces: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn get_tree() -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let tree: Node = conn
        .get_tree()
        .await
        .map_err(|e| internal_error(format!("Failed to get tree: {}", e)))?;

    let json = serde_json::to_string_pretty(&tree)
        .map_err(|e| internal_error(format!("Failed to serialize tree: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn switch_workspace(params: SwitchWorkspaceParams) -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let command = format!("workspace {}", params.workspace);
    let results = conn
        .run_command(&command)
        .await
        .map_err(|e| internal_error(format!("Failed to switch workspace: {}", e)))?;

    let success = results.iter().all(|r| r.success);
    if success {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Switched to workspace '{}'",
            params.workspace
        ))]))
    } else {
        let errors: Vec<String> = results.iter().filter_map(|r| r.error.clone()).collect();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to switch workspace: {}",
            errors.join(", ")
        ))]))
    }
}

pub async fn focus_window(params: FocusWindowParams) -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let command = format!("{} focus", params.criteria);
    let results = conn
        .run_command(&command)
        .await
        .map_err(|e| internal_error(format!("Failed to focus window: {}", e)))?;

    let success = results.iter().all(|r| r.success);
    if success {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Focused window matching '{}'",
            params.criteria
        ))]))
    } else {
        let errors: Vec<String> = results.iter().filter_map(|r| r.error.clone()).collect();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to focus window: {}",
            errors.join(", ")
        ))]))
    }
}

pub async fn move_to_workspace(params: MoveToWorkspaceParams) -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let command = format!("move container to workspace {}", params.workspace);
    let results = conn
        .run_command(&command)
        .await
        .map_err(|e| internal_error(format!("Failed to move window: {}", e)))?;

    let success = results.iter().all(|r| r.success);
    if success {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Moved window to workspace '{}'",
            params.workspace
        ))]))
    } else {
        let errors: Vec<String> = results.iter().filter_map(|r| r.error.clone()).collect();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to move window: {}",
            errors.join(", ")
        ))]))
    }
}

pub async fn run_command(params: RunCommandParams) -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let results = conn
        .run_command(&params.command)
        .await
        .map_err(|e| internal_error(format!("Failed to run command: {}", e)))?;

    let json = serde_json::to_string_pretty(&results)
        .map_err(|e| internal_error(format!("Failed to serialize results: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn exec(params: ExecParams) -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let command = format!("exec {}", params.command);
    let results = conn
        .run_command(&command)
        .await
        .map_err(|e| internal_error(format!("Failed to exec: {}", e)))?;

    let success = results.iter().all(|r| r.success);
    if success {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Launched '{}'",
            params.command
        ))]))
    } else {
        let errors: Vec<String> = results.iter().filter_map(|r| r.error.clone()).collect();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to launch: {}",
            errors.join(", ")
        ))]))
    }
}

pub async fn kill() -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let results = conn
        .run_command("kill")
        .await
        .map_err(|e| internal_error(format!("Failed to kill window: {}", e)))?;

    let success = results.iter().all(|r| r.success);
    if success {
        Ok(CallToolResult::success(vec![Content::text(
            "Killed focused window",
        )]))
    } else {
        let errors: Vec<String> = results.iter().filter_map(|r| r.error.clone()).collect();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to kill window: {}",
            errors.join(", ")
        ))]))
    }
}

pub async fn kill_window(params: KillWindowParams) -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let command = format!("{} kill", params.criteria);
    let results = conn
        .run_command(&command)
        .await
        .map_err(|e| internal_error(format!("Failed to kill window: {}", e)))?;

    let success = results.iter().all(|r| r.success);
    if success {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Killed window matching '{}'",
            params.criteria
        ))]))
    } else {
        let errors: Vec<String> = results.iter().filter_map(|r| r.error.clone()).collect();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to kill window: {}",
            errors.join(", ")
        ))]))
    }
}

pub async fn fullscreen() -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let results = conn
        .run_command("fullscreen toggle")
        .await
        .map_err(|e| internal_error(format!("Failed to toggle fullscreen: {}", e)))?;

    let success = results.iter().all(|r| r.success);
    if success {
        Ok(CallToolResult::success(vec![Content::text(
            "Toggled fullscreen",
        )]))
    } else {
        let errors: Vec<String> = results.iter().filter_map(|r| r.error.clone()).collect();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to toggle fullscreen: {}",
            errors.join(", ")
        ))]))
    }
}

pub async fn get_outputs() -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let outputs = conn
        .get_outputs()
        .await
        .map_err(|e| internal_error(format!("Failed to get outputs: {}", e)))?;

    let json = serde_json::to_string_pretty(&outputs)
        .map_err(|e| internal_error(format!("Failed to serialize outputs: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn get_marks() -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let marks = conn
        .get_marks()
        .await
        .map_err(|e| internal_error(format!("Failed to get marks: {}", e)))?;

    if marks.0.is_empty() {
        Ok(CallToolResult::success(vec![Content::text(
            "No marks defined",
        )]))
    } else {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Marks:\n{}",
            marks.0.join("\n")
        ))]))
    }
}

pub async fn get_binding_modes() -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let modes = conn
        .get_binding_modes()
        .await
        .map_err(|e| internal_error(format!("Failed to get binding modes: {}", e)))?;

    let json = serde_json::to_string_pretty(&modes)
        .map_err(|e| internal_error(format!("Failed to serialize binding modes: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn get_version() -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let version = conn
        .get_version()
        .await
        .map_err(|e| internal_error(format!("Failed to get version: {}", e)))?;

    let json = serde_json::to_string_pretty(&version)
        .map_err(|e| internal_error(format!("Failed to serialize version: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn get_scratchpad() -> Result<CallToolResult, McpError> {
    let mut conn = connect().await?;

    let tree = conn
        .get_tree()
        .await
        .map_err(|e| internal_error(format!("Failed to get tree: {}", e)))?;

    fn find_scratchpad_windows(node: &Node) -> Vec<String> {
        let mut windows = Vec::new();

        if node.name.as_deref() == Some("__i3_scratch") {
            fn collect_windows(n: &Node, windows: &mut Vec<String>) {
                if let Some(ref name) = n.name {
                    if n.window.is_some() {
                        windows.push(format!("{} (id: {})", name, n.id));
                    }
                }
                for child in &n.nodes {
                    collect_windows(child, windows);
                }
                for child in &n.floating_nodes {
                    collect_windows(child, windows);
                }
            }
            collect_windows(node, &mut windows);
        }

        for child in &node.nodes {
            windows.extend(find_scratchpad_windows(child));
        }

        windows
    }

    let scratchpad_windows = find_scratchpad_windows(&tree);

    if scratchpad_windows.is_empty() {
        Ok(CallToolResult::success(vec![Content::text(
            "Scratchpad is empty",
        )]))
    } else {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Scratchpad windows ({}):\n{}",
            scratchpad_windows.len(),
            scratchpad_windows.join("\n")
        ))]))
    }
}
