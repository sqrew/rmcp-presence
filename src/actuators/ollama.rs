//! Ollama local LLM management actuators

use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const DEFAULT_HOST: &str = "http://localhost:11434";

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HostParams {
    #[schemars(description = "Ollama host URL (default: http://localhost:11434)")]
    pub host: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ModelParams {
    #[schemars(description = "Model name (e.g., \"llama3.2\", \"mistral\", \"codellama:7b\")")]
    pub name: String,
    #[schemars(description = "Ollama host URL (default: http://localhost:11434)")]
    pub host: Option<String>,
}

// === API Response Types ===

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Option<Vec<ModelInfo>>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    name: String,
    size: Option<u64>,
    #[allow(dead_code)]
    digest: Option<String>,
    #[allow(dead_code)]
    modified_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PsResponse {
    models: Option<Vec<RunningModel>>,
}

#[derive(Debug, Deserialize)]
struct RunningModel {
    name: String,
    size: Option<u64>,
    size_vram: Option<u64>,
    #[allow(dead_code)]
    expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct ModelRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ShowResponse {
    #[allow(dead_code)]
    modelfile: Option<String>,
    parameters: Option<String>,
    template: Option<String>,
    #[allow(dead_code)]
    license: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PullResponse {
    status: Option<String>,
}

// === Helper Functions ===

fn get_host(host: Option<&str>) -> String {
    host.unwrap_or(DEFAULT_HOST).to_string()
}

fn format_size(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

async fn get_client() -> Result<reqwest::Client, McpError> {
    Ok(reqwest::Client::new())
}

// === Tool Functions ===

pub async fn list_models(params: HostParams) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let host = get_host(params.host.as_deref());
    let url = format!("{}/api/tags", host);

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to connect to Ollama at {}: {}",
                host, e
            ))]))
        }
    };

    if !response.status().is_success() {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "Ollama returned error: {}",
            response.status()
        ))]));
    }

    let tags: TagsResponse = match response.json().await {
        Ok(t) => t,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to parse response: {}",
                e
            ))]))
        }
    };

    let models = tags.models.unwrap_or_default();
    if models.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No models installed. Use pull_model to download one.",
        )]));
    }

    let mut output = format!("{} model(s) installed:\n\n", models.len());
    for model in models {
        let size = model.size.map(format_size).unwrap_or_else(|| "?".into());
        output.push_str(&format!("  {} ({})\n", model.name, size));
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn list_running(params: HostParams) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let host = get_host(params.host.as_deref());
    let url = format!("{}/api/ps", host);

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to connect to Ollama at {}: {}",
                host, e
            ))]))
        }
    };

    if !response.status().is_success() {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "Ollama returned error: {}",
            response.status()
        ))]));
    }

    let ps: PsResponse = match response.json().await {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to parse response: {}",
                e
            ))]))
        }
    };

    let models = ps.models.unwrap_or_default();
    if models.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No models currently loaded in memory.",
        )]));
    }

    let mut output = format!("{} model(s) loaded:\n\n", models.len());
    for model in models {
        let size = model.size.map(format_size).unwrap_or_else(|| "?".into());
        let vram = model
            .size_vram
            .map(|v| format!(" (VRAM: {})", format_size(v)))
            .unwrap_or_default();
        output.push_str(&format!("  {} - {}{}\n", model.name, size, vram));
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn show_model(params: ModelParams) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let host = get_host(params.host.as_deref());
    let url = format!("{}/api/show", host);

    let request = ModelRequest {
        name: params.name.clone(),
        stream: None,
    };

    let response = match client.post(&url).json(&request).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to connect to Ollama: {}",
                e
            ))]))
        }
    };

    if !response.status().is_success() {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "Model '{}' not found or error: {}",
            params.name,
            response.status()
        ))]));
    }

    let show: ShowResponse = match response.json().await {
        Ok(s) => s,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to parse response: {}",
                e
            ))]))
        }
    };

    let mut output = format!("Model: {}\n\n", params.name);

    if let Some(parameters) = show.parameters {
        output.push_str(&format!("Parameters:\n{}\n\n", parameters));
    }

    if let Some(template) = show.template {
        let preview = if template.len() > 200 {
            format!("{}...", &template[..200])
        } else {
            template
        };
        output.push_str(&format!("Template:\n{}\n", preview));
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn pull_model(params: ModelParams) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let host = get_host(params.host.as_deref());
    let url = format!("{}/api/pull", host);

    let request = ModelRequest {
        name: params.name.clone(),
        stream: Some(false),
    };

    let response = match client.post(&url).json(&request).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to connect to Ollama: {}",
                e
            ))]))
        }
    };

    if !response.status().is_success() {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to pull '{}': {}",
            params.name,
            response.status()
        ))]));
    }

    let pull: PullResponse = match response.json().await {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Pull started but failed to parse response: {}",
                e
            ))]))
        }
    };

    let status = pull.status.unwrap_or_else(|| "completed".into());
    Ok(CallToolResult::success(vec![Content::text(format!(
        "Pull '{}': {}",
        params.name, status
    ))]))
}

pub async fn delete_model(params: ModelParams) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let host = get_host(params.host.as_deref());
    let url = format!("{}/api/delete", host);

    let request = ModelRequest {
        name: params.name.clone(),
        stream: None,
    };

    let response = match client.delete(&url).json(&request).send().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to connect to Ollama: {}",
                e
            ))]))
        }
    };

    if response.status().is_success() {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Deleted model '{}'",
            params.name
        ))]))
    } else {
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to delete '{}': {}",
            params.name,
            response.status()
        ))]))
    }
}

// === Composite Types ===

#[derive(Debug, Serialize)]
pub struct OllamaStatus {
    pub online: bool,
    pub host: String,
    pub installed_models: Vec<InstalledModel>,
    pub running_models: Vec<LoadedModel>,
    pub installed_count: usize,
    pub running_count: usize,
}

#[derive(Debug, Serialize)]
pub struct InstalledModel {
    pub name: String,
    pub size: String,
}

#[derive(Debug, Serialize)]
pub struct LoadedModel {
    pub name: String,
    pub size: String,
    pub vram: Option<String>,
}

// === Composite Function ===

pub async fn get_ollama_status(params: HostParams) -> Result<CallToolResult, McpError> {
    let client = match get_client().await {
        Ok(c) => c,
        Err(_) => {
            let status = OllamaStatus {
                online: false,
                host: get_host(params.host.as_deref()),
                installed_models: vec![],
                running_models: vec![],
                installed_count: 0,
                running_count: 0,
            };
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&status).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            )]));
        }
    };

    let host = get_host(params.host.as_deref());

    // Check if Ollama is reachable and get installed models
    let tags_url = format!("{}/api/tags", host);
    let installed_models = match client.get(&tags_url).send().await {
        Ok(response) if response.status().is_success() => {
            match response.json::<TagsResponse>().await {
                Ok(tags) => tags.models.unwrap_or_default()
                    .into_iter()
                    .map(|m| InstalledModel {
                        name: m.name,
                        size: m.size.map(format_size).unwrap_or_else(|| "?".into()),
                    })
                    .collect(),
                Err(_) => vec![],
            }
        }
        _ => {
            // Ollama not reachable
            let status = OllamaStatus {
                online: false,
                host,
                installed_models: vec![],
                running_models: vec![],
                installed_count: 0,
                running_count: 0,
            };
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&status).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            )]));
        }
    };

    // Get running models
    let ps_url = format!("{}/api/ps", host);
    let running_models: Vec<LoadedModel> = match client.get(&ps_url).send().await {
        Ok(response) if response.status().is_success() => {
            match response.json::<PsResponse>().await {
                Ok(ps) => ps.models.unwrap_or_default()
                    .into_iter()
                    .map(|m| LoadedModel {
                        name: m.name,
                        size: m.size.map(format_size).unwrap_or_else(|| "?".into()),
                        vram: m.size_vram.map(format_size),
                    })
                    .collect(),
                Err(_) => vec![],
            }
        }
        _ => vec![],
    };

    let status = OllamaStatus {
        online: true,
        host,
        installed_count: installed_models.len(),
        running_count: running_models.len(),
        installed_models,
        running_models,
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&status).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
    )]))
}
