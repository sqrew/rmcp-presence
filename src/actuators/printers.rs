//! Printer management actuators

use printers::{
    common::base::{job::PrinterJobOptions, printer::PrinterState},
    get_default_printer, get_printer_by_name, get_printers,
};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrinterNameParams {
    #[schemars(description = "Name of the printer")]
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrintFileParams {
    #[schemars(description = "Name of the printer (uses default if not specified)")]
    pub printer: Option<String>,
    #[schemars(description = "Path to the file to print")]
    pub file_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrintTextParams {
    #[schemars(description = "Name of the printer (uses default if not specified)")]
    pub printer: Option<String>,
    #[schemars(description = "Text content to print")]
    pub text: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JobParams {
    #[schemars(description = "Name of the printer")]
    pub printer: String,
    #[schemars(description = "Job ID")]
    pub job_id: u64,
}

// === Helper Functions ===

fn state_to_string(state: &PrinterState) -> &'static str {
    match state {
        PrinterState::READY => "ready",
        PrinterState::OFFLINE => "offline",
        PrinterState::PAUSED => "paused",
        PrinterState::PRINTING => "printing",
        PrinterState::UNKNOWN => "unknown",
    }
}

// === Tool Functions ===

pub async fn list_printers() -> Result<CallToolResult, McpError> {
    let printers = get_printers();

    let output = if printers.is_empty() {
        "No printers found.".to_string()
    } else {
        let mut result = format!("Found {} printer(s):\n\n", printers.len());
        for p in &printers {
            result.push_str(&format!(
                "• {} {}\n  System: {}\n  Driver: {}\n  URI: {}\n  State: {}\n\n",
                p.name,
                if p.is_default { "(default)" } else { "" },
                p.system_name,
                p.driver_name,
                p.uri,
                state_to_string(&p.state)
            ));
        }
        result
    };

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn get_printer_info(params: PrinterNameParams) -> Result<CallToolResult, McpError> {
    match get_printer_by_name(&params.name) {
        Some(p) => {
            let output = format!(
                "Printer: {} {}\nSystem Name: {}\nDriver: {}\nURI: {}\nLocation: {}\nState: {}\nShared: {}",
                p.name,
                if p.is_default { "(default)" } else { "" },
                p.system_name,
                p.driver_name,
                p.uri,
                p.location,
                state_to_string(&p.state),
                p.is_shared
            );
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "Printer '{}' not found.",
            params.name
        ))])),
    }
}

pub async fn get_default_printer_fn() -> Result<CallToolResult, McpError> {
    match get_default_printer() {
        Some(p) => {
            let output = format!(
                "Default Printer: {}\nSystem Name: {}\nDriver: {}\nURI: {}\nState: {}",
                p.name,
                p.system_name,
                p.driver_name,
                p.uri,
                state_to_string(&p.state)
            );
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(
            "No default printer configured.",
        )])),
    }
}

pub async fn print_file(params: PrintFileParams) -> Result<CallToolResult, McpError> {
    let printer = match &params.printer {
        Some(name) => get_printer_by_name(name),
        None => get_default_printer(),
    };

    match printer {
        Some(p) => match p.print_file(&params.file_path, PrinterJobOptions::none()) {
            Ok(job_id) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Print job submitted successfully.\nJob ID: {}\nPrinter: {}\nFile: {}",
                job_id, p.name, params.file_path
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to print file: {}",
                e
            ))])),
        },
        None => {
            let msg = match &params.printer {
                Some(name) => format!("Printer '{}' not found.", name),
                None => "No default printer configured.".to_string(),
            };
            Ok(CallToolResult::success(vec![Content::text(msg)]))
        }
    }
}

pub async fn print_text(params: PrintTextParams) -> Result<CallToolResult, McpError> {
    let printer = match &params.printer {
        Some(name) => get_printer_by_name(name),
        None => get_default_printer(),
    };

    match printer {
        Some(p) => match p.print(params.text.as_bytes(), PrinterJobOptions::none()) {
            Ok(job_id) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Print job submitted successfully.\nJob ID: {}\nPrinter: {}\nContent length: {} bytes",
                job_id, p.name, params.text.len()
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to print text: {}",
                e
            ))])),
        },
        None => {
            let msg = match &params.printer {
                Some(name) => format!("Printer '{}' not found.", name),
                None => "No default printer configured.".to_string(),
            };
            Ok(CallToolResult::success(vec![Content::text(msg)]))
        }
    }
}

pub async fn list_jobs(params: PrinterNameParams) -> Result<CallToolResult, McpError> {
    match get_printer_by_name(&params.name) {
        Some(p) => {
            let jobs = p.get_active_jobs();
            if jobs.is_empty() {
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "No active jobs on printer '{}'.",
                    params.name
                ))]))
            } else {
                let mut result = format!("Active jobs on '{}':\n\n", params.name);
                for job in &jobs {
                    result.push_str(&format!("• Job {}: {} ({:?})\n", job.id, job.name, job.state));
                }
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
        }
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "Printer '{}' not found.",
            params.name
        ))])),
    }
}

pub async fn cancel_job(params: JobParams) -> Result<CallToolResult, McpError> {
    match get_printer_by_name(&params.printer) {
        Some(p) => match p.cancel_job(params.job_id) {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Job {} cancelled on printer '{}'.",
                params.job_id, params.printer
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to cancel job: {}",
                e
            ))])),
        },
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "Printer '{}' not found.",
            params.printer
        ))])),
    }
}

pub async fn pause_job(params: JobParams) -> Result<CallToolResult, McpError> {
    match get_printer_by_name(&params.printer) {
        Some(p) => match p.pause_job(params.job_id) {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Job {} paused on printer '{}'.",
                params.job_id, params.printer
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to pause job: {}",
                e
            ))])),
        },
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "Printer '{}' not found.",
            params.printer
        ))])),
    }
}

pub async fn resume_job(params: JobParams) -> Result<CallToolResult, McpError> {
    match get_printer_by_name(&params.printer) {
        Some(p) => match p.resume_job(params.job_id) {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Job {} resumed on printer '{}'.",
                params.job_id, params.printer
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to resume job: {}",
                e
            ))])),
        },
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "Printer '{}' not found.",
            params.printer
        ))])),
    }
}

pub async fn restart_job(params: JobParams) -> Result<CallToolResult, McpError> {
    match get_printer_by_name(&params.printer) {
        Some(p) => match p.restart_job(params.job_id) {
            Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Job {} restarted on printer '{}'.",
                params.job_id, params.printer
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to restart job: {}",
                e
            ))])),
        },
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "Printer '{}' not found.",
            params.printer
        ))])),
    }
}
