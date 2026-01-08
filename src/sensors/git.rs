//! Git repository sensors

use crate::shared::internal_error;
use git2::{BranchType, Repository, StatusOptions};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepoPathParams {
    #[schemars(description = "Path to the git repository (defaults to current directory)")]
    pub path: Option<String>,
}

// === Helper Functions ===

fn get_repo(path: Option<String>) -> Result<Repository, McpError> {
    let repo_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    Repository::discover(&repo_path)
        .map_err(|e| internal_error(format!("Not a git repository: {}", e)))
}

// === Tool Functions ===

pub async fn get_status(params: RepoPathParams) -> Result<CallToolResult, McpError> {
    let repo = get_repo(params.path)?;
    let mut result = String::from("Git Repository Status:\n\n");

    if let Some(workdir) = repo.workdir() {
        result.push_str(&format!("Repository: {}\n", workdir.display()));
    }

    match repo.head() {
        Ok(head) => {
            if let Some(name) = head.shorthand() {
                result.push_str(&format!("Branch: {}\n", name));
            }

            if let Ok(commit) = head.peel_to_commit() {
                let id = commit.id();
                let short_id = &id.to_string()[..7];
                let summary = commit.summary().unwrap_or("(no message)");
                let time = commit.time();
                let timestamp = chrono::DateTime::from_timestamp(time.seconds(), 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                result.push_str("\nLast Commit:\n");
                result.push_str(&format!("  {} - {}\n", short_id, summary));
                result.push_str(&format!(
                    "  Author: {}\n",
                    commit.author().name().unwrap_or("unknown")
                ));
                result.push_str(&format!("  Date: {}\n", timestamp));
            }
        }
        Err(_) => {
            result.push_str("Branch: (no commits yet)\n");
        }
    }

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);

    match repo.statuses(Some(&mut opts)) {
        Ok(statuses) => {
            let mut staged = Vec::new();
            let mut modified = Vec::new();
            let mut untracked = Vec::new();

            for entry in statuses.iter() {
                let path = entry.path().unwrap_or("?");
                let status = entry.status();

                if status.is_index_new()
                    || status.is_index_modified()
                    || status.is_index_deleted()
                {
                    staged.push(path.to_string());
                }
                if status.is_wt_modified() || status.is_wt_deleted() {
                    modified.push(path.to_string());
                }
                if status.is_wt_new() {
                    untracked.push(path.to_string());
                }
            }

            result.push_str("\nWorking Tree:\n");

            if staged.is_empty() && modified.is_empty() && untracked.is_empty() {
                result.push_str("  Clean - nothing to commit\n");
            } else {
                if !staged.is_empty() {
                    result.push_str(&format!("  Staged: {} file(s)\n", staged.len()));
                    for f in staged.iter().take(5) {
                        result.push_str(&format!("    + {}\n", f));
                    }
                    if staged.len() > 5 {
                        result.push_str(&format!("    ... and {} more\n", staged.len() - 5));
                    }
                }
                if !modified.is_empty() {
                    result.push_str(&format!("  Modified: {} file(s)\n", modified.len()));
                    for f in modified.iter().take(5) {
                        result.push_str(&format!("    M {}\n", f));
                    }
                    if modified.len() > 5 {
                        result.push_str(&format!("    ... and {} more\n", modified.len() - 5));
                    }
                }
                if !untracked.is_empty() {
                    result.push_str(&format!("  Untracked: {} file(s)\n", untracked.len()));
                    for f in untracked.iter().take(5) {
                        result.push_str(&format!("    ? {}\n", f));
                    }
                    if untracked.len() > 5 {
                        result.push_str(&format!("    ... and {} more\n", untracked.len() - 5));
                    }
                }
            }
        }
        Err(e) => {
            result.push_str(&format!("\nCould not get status: {}\n", e));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_log(params: RepoPathParams) -> Result<CallToolResult, McpError> {
    let repo = get_repo(params.path)?;
    let mut result = String::from("Recent Commits:\n\n");

    let head = repo
        .head()
        .map_err(|e| internal_error(format!("No HEAD: {}", e)))?;

    let oid = head
        .target()
        .ok_or_else(|| internal_error("HEAD has no target"))?;

    let mut revwalk = repo
        .revwalk()
        .map_err(|e| internal_error(format!("Failed to create revwalk: {}", e)))?;

    revwalk
        .push(oid)
        .map_err(|e| internal_error(format!("Failed to push HEAD: {}", e)))?;

    let mut count = 0;
    for oid in revwalk.take(10) {
        if let Ok(oid) = oid {
            if let Ok(commit) = repo.find_commit(oid) {
                count += 1;
                let id_str = oid.to_string();
                let short_id = &id_str[..7];
                let summary = commit.summary().unwrap_or("(no message)").to_string();
                let author = commit.author();
                let author_name = author.name().unwrap_or("unknown");

                result.push_str(&format!("{} {} - {}\n", short_id, author_name, summary));
            }
        }
    }

    if count == 0 {
        result.push_str("No commits found.\n");
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_current_branch(params: RepoPathParams) -> Result<CallToolResult, McpError> {
    let repo = get_repo(params.path)?;

    let head = repo
        .head()
        .map_err(|e| internal_error(format!("No HEAD: {}", e)))?;

    let branch_name = head.shorthand().unwrap_or("(detached)");
    let is_detached = repo.head_detached().unwrap_or(false);

    let result = if is_detached {
        format!("Current branch: {} (detached HEAD)", branch_name)
    } else {
        format!("Current branch: {}", branch_name)
    };

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_branches(params: RepoPathParams) -> Result<CallToolResult, McpError> {
    let repo = get_repo(params.path)?;
    let mut result = String::from("Branches:\n\n");

    let current = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from));

    result.push_str("Local:\n");
    let local_branches = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| internal_error(format!("Failed to list branches: {}", e)))?;

    let mut local_count = 0;
    for branch in local_branches {
        if let Ok((branch, _)) = branch {
            if let Ok(Some(name)) = branch.name() {
                local_count += 1;
                let marker = if Some(name.to_string()) == current {
                    "* "
                } else {
                    "  "
                };
                result.push_str(&format!("{}{}\n", marker, name));
            }
        }
    }
    if local_count == 0 {
        result.push_str("  (none)\n");
    }

    result.push_str("\nRemote:\n");
    let remote_branches = repo
        .branches(Some(BranchType::Remote))
        .map_err(|e| internal_error(format!("Failed to list remote branches: {}", e)))?;

    let mut remote_count = 0;
    for branch in remote_branches {
        if let Ok((branch, _)) = branch {
            if let Ok(Some(name)) = branch.name() {
                remote_count += 1;
                result.push_str(&format!("  {}\n", name));
            }
        }
    }
    if remote_count == 0 {
        result.push_str("  (none)\n");
    }

    result.push_str(&format!(
        "\nTotal: {} local, {} remote\n",
        local_count, remote_count
    ));

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_remotes(params: RepoPathParams) -> Result<CallToolResult, McpError> {
    let repo = get_repo(params.path)?;
    let mut result = String::from("Remotes:\n\n");

    let remotes = repo
        .remotes()
        .map_err(|e| internal_error(format!("Failed to list remotes: {}", e)))?;

    if remotes.is_empty() {
        result.push_str("No remotes configured.\n");
    } else {
        for name in remotes.iter().flatten() {
            result.push_str(&format!("{}:\n", name));
            if let Ok(remote) = repo.find_remote(name) {
                if let Some(url) = remote.url() {
                    result.push_str(&format!("  Fetch: {}\n", url));
                }
                if let Some(url) = remote.pushurl().or(remote.url()) {
                    result.push_str(&format!("  Push:  {}\n", url));
                }
            }
            result.push('\n');
        }
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_tags(params: RepoPathParams) -> Result<CallToolResult, McpError> {
    let repo = get_repo(params.path)?;
    let mut result = String::from("Tags:\n\n");

    let tags = repo
        .tag_names(None)
        .map_err(|e| internal_error(format!("Failed to list tags: {}", e)))?;

    if tags.is_empty() {
        result.push_str("No tags found.\n");
    } else {
        for tag in tags.iter().flatten() {
            result.push_str(&format!("  {}\n", tag));
        }
        result.push_str(&format!("\nTotal: {} tags\n", tags.len()));
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_stash_list(params: RepoPathParams) -> Result<CallToolResult, McpError> {
    let mut repo = get_repo(params.path)?;
    let mut result = String::from("Stash List:\n\n");

    let mut stashes = Vec::new();
    repo.stash_foreach(|index, message, _oid| {
        stashes.push((index, message.to_string()));
        true
    })
    .map_err(|e| internal_error(format!("Failed to list stashes: {}", e)))?;

    if stashes.is_empty() {
        result.push_str("No stashed changes.\n");
    } else {
        for (index, message) in &stashes {
            result.push_str(&format!("stash@{{{}}}: {}\n", index, message));
        }
        result.push_str(&format!("\nTotal: {} stash entries\n", stashes.len()));
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_diff_summary(params: RepoPathParams) -> Result<CallToolResult, McpError> {
    let repo = get_repo(params.path)?;
    let mut result = String::from("Diff Summary:\n\n");

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    let statuses = repo
        .statuses(Some(&mut opts))
        .map_err(|e| internal_error(format!("Failed to get status: {}", e)))?;

    let mut staged_new = 0;
    let mut staged_modified = 0;
    let mut staged_deleted = 0;
    let mut unstaged_modified = 0;
    let mut unstaged_deleted = 0;
    let mut untracked = 0;

    for entry in statuses.iter() {
        let status = entry.status();

        if status.is_index_new() {
            staged_new += 1;
        }
        if status.is_index_modified() {
            staged_modified += 1;
        }
        if status.is_index_deleted() {
            staged_deleted += 1;
        }
        if status.is_wt_modified() {
            unstaged_modified += 1;
        }
        if status.is_wt_deleted() {
            unstaged_deleted += 1;
        }
        if status.is_wt_new() {
            untracked += 1;
        }
    }

    let staged_total = staged_new + staged_modified + staged_deleted;
    let unstaged_total = unstaged_modified + unstaged_deleted;

    result.push_str("Staged for commit:\n");
    if staged_total == 0 {
        result.push_str("  (none)\n");
    } else {
        if staged_new > 0 {
            result.push_str(&format!("  {} new file(s)\n", staged_new));
        }
        if staged_modified > 0 {
            result.push_str(&format!("  {} modified\n", staged_modified));
        }
        if staged_deleted > 0 {
            result.push_str(&format!("  {} deleted\n", staged_deleted));
        }
    }

    result.push_str("\nNot staged:\n");
    if unstaged_total == 0 {
        result.push_str("  (none)\n");
    } else {
        if unstaged_modified > 0 {
            result.push_str(&format!("  {} modified\n", unstaged_modified));
        }
        if unstaged_deleted > 0 {
            result.push_str(&format!("  {} deleted\n", unstaged_deleted));
        }
    }

    result.push_str("\nUntracked:\n");
    if untracked == 0 {
        result.push_str("  (none)\n");
    } else {
        result.push_str(&format!("  {} file(s)\n", untracked));
    }

    result.push_str(&format!(
        "\nSummary: {} staged, {} unstaged, {} untracked\n",
        staged_total, unstaged_total, untracked
    ));

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
