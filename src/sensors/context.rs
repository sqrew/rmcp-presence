//! Context - composite environmental awareness in one call

use rmcp::{model::{CallToolResult, Content}, ErrorData as McpError};
use serde::Serialize;

use crate::shared::internal_error;

#[derive(Debug, Serialize)]
pub struct Context {
    // Time
    pub datetime_local: String,
    pub datetime_utc: String,
    pub timezone: String,

    // User
    pub user: String,
    pub home: String,
    pub shell: Option<String>,
    pub editor: Option<String>,
    pub locale: Option<String>,

    // Desktop (Linux)
    pub desktop: Option<String>,
    pub display_server: Option<String>,

    // State
    pub idle_seconds: Option<u64>,

    // System summary
    pub cpu_usage_percent: Option<f32>,
    pub memory_used_gb: Option<f32>,
    pub memory_total_gb: Option<f32>,

    // Battery (if present)
    pub battery_percent: Option<f32>,
    pub battery_charging: Option<bool>,
}

pub async fn get_context() -> Result<CallToolResult, McpError> {
    let context = build_context().map_err(|e| internal_error(e.to_string()))?;
    let json = serde_json::to_string_pretty(&context).map_err(|e| internal_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

fn build_context() -> anyhow::Result<Context> {
    // Time
    let now_local = chrono::Local::now();
    let now_utc = chrono::Utc::now();
    let datetime_local = now_local.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
    let datetime_utc = now_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let timezone = now_local.format("%Z").to_string();

    // User info from env
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "unknown".to_string());

    let shell = std::env::var("SHELL").ok();
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .ok();
    let locale = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .ok();

    // Desktop environment (Linux)
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok();

    let display_server = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Some("wayland".to_string())
    } else if std::env::var("DISPLAY").is_ok() {
        Some("x11".to_string())
    } else {
        None
    };

    // Idle time
    let idle_seconds = user_idle::UserIdle::get_time()
        .ok()
        .map(|d| d.as_seconds());

    // System stats
    let mut sys = sysinfo::System::new();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    // Small delay to get accurate CPU reading
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_cpu_usage();

    let cpu_usage_percent = Some(sys.global_cpu_usage());
    let memory_used_gb = Some(sys.used_memory() as f32 / 1_073_741_824.0);
    let memory_total_gb = Some(sys.total_memory() as f32 / 1_073_741_824.0);

    // Battery
    let (battery_percent, battery_charging) = get_battery_info();

    Ok(Context {
        datetime_local,
        datetime_utc,
        timezone,
        user,
        home,
        shell,
        editor,
        locale,
        desktop,
        display_server,
        idle_seconds,
        cpu_usage_percent,
        memory_used_gb,
        memory_total_gb,
        battery_percent,
        battery_charging,
    })
}

fn get_battery_info() -> (Option<f32>, Option<bool>) {
    let manager = match battery::Manager::new() {
        Ok(m) => m,
        Err(_) => return (None, None),
    };

    let mut batteries = match manager.batteries() {
        Ok(b) => b,
        Err(_) => return (None, None),
    };

    if let Some(Ok(battery)) = batteries.next() {
        let percent = battery.state_of_charge().value * 100.0;
        let charging = matches!(
            battery.state(),
            battery::State::Charging | battery::State::Full
        );
        (Some(percent), Some(charging))
    } else {
        (None, None)
    }
}
