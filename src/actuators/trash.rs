//! Trash/recycle bin actuators

use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrashFileParams {
    #[schemars(description = "Path to the file or directory to move to trash")]
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TrashFilesParams {
    #[schemars(description = "List of paths to move to trash")]
    pub paths: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RestoreParams {
    #[schemars(description = "Name of the file to restore from trash (partial match supported)")]
    pub name: String,
}

// === Tool Functions ===

pub async fn trash_file(params: TrashFileParams) -> Result<CallToolResult, McpError> {
    let path = PathBuf::from(&params.path);

    if !path.exists() {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "Path does not exist: {}",
            params.path
        ))]));
    }

    match trash::delete(&path) {
        Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Moved to trash: {}",
            params.path
        ))])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to trash: {}",
            e
        ))])),
    }
}

pub async fn trash_files(params: TrashFilesParams) -> Result<CallToolResult, McpError> {
    let paths: Vec<PathBuf> = params.paths.iter().map(PathBuf::from).collect();

    let mut missing: Vec<&str> = Vec::new();
    let mut to_trash: Vec<&PathBuf> = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        if path.exists() {
            to_trash.push(path);
        } else {
            missing.push(&params.paths[i]);
        }
    }

    if to_trash.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No valid paths to trash",
        )]));
    }

    match trash::delete_all(&to_trash) {
        Ok(()) => {
            let mut msg = format!("Moved {} items to trash", to_trash.len());
            if !missing.is_empty() {
                msg.push_str(&format!("\nSkipped (not found): {}", missing.join(", ")));
            }
            Ok(CallToolResult::success(vec![Content::text(msg)]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to trash: {}",
            e
        ))])),
    }
}

pub async fn list_trash() -> Result<CallToolResult, McpError> {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        match trash::os_limited::list() {
            Ok(items) => {
                if items.is_empty() {
                    Ok(CallToolResult::success(vec![Content::text("Trash is empty")]))
                } else {
                    let list: Vec<String> = items
                        .iter()
                        .map(|item| item.name.to_string_lossy().into_owned())
                        .collect();
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Trash contents ({} items):\n{}",
                        items.len(),
                        list.join("\n")
                    ))]))
                }
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list trash: {}",
                e
            ))])),
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Ok(CallToolResult::success(vec![Content::text(
            "list_trash is not supported on this platform (Linux/Windows only)",
        )]))
    }
}

pub async fn restore_from_trash(params: RestoreParams) -> Result<CallToolResult, McpError> {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        match trash::os_limited::list() {
            Ok(items) => {
                let search = params.name.to_lowercase();
                let matches: Vec<_> = items
                    .into_iter()
                    .filter(|item| item.name.to_string_lossy().to_lowercase().contains(&search))
                    .collect();

                if matches.is_empty() {
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "No items in trash matching '{}'",
                        params.name
                    ))]));
                }

                let count = matches.len();
                let names: Vec<String> = matches
                    .iter()
                    .map(|item| item.name.to_string_lossy().into_owned())
                    .collect();

                match trash::os_limited::restore_all(matches) {
                    Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                        "Restored {} item(s): {}",
                        count,
                        names.join(", ")
                    ))])),
                    Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                        "Failed to restore: {}",
                        e
                    ))])),
                }
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list trash: {}",
                e
            ))])),
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Ok(CallToolResult::success(vec![Content::text(
            "restore_from_trash is not supported on this platform (Linux/Windows only)",
        )]))
    }
}

pub async fn empty_trash() -> Result<CallToolResult, McpError> {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        match trash::os_limited::list() {
            Ok(items) => {
                if items.is_empty() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        "Trash is already empty",
                    )]));
                }

                let count = items.len();
                match trash::os_limited::purge_all(items) {
                    Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                        "Permanently deleted {} item(s) from trash",
                        count
                    ))])),
                    Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                        "Failed to empty trash: {}",
                        e
                    ))])),
                }
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to list trash: {}",
                e
            ))])),
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Ok(CallToolResult::success(vec![Content::text(
            "empty_trash is not supported on this platform (Linux/Windows only)",
        )]))
    }
}
