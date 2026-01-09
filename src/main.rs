//! rmcp-presence: Unified MCP server for AI environmental awareness
//!
//! One binary. 150 tools. Cross-platform base with Linux power features.
//!
//! Features:
//! - sensors: System info, display, idle, network, USB, battery, bluetooth, git, weather (29 tools)
//! - actuators: Clipboard, audio, trash, open, screenshot, camera, microphone, ollama (38 tools)
//! - linux: i3, xdotool, mpris, systemd, brightness, bluer, dbus, logind, pulseaudio (83 tools)

use clap::{Parser, Subcommand};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters, ServerHandler},
    model::*,
    ErrorData as McpError,
    ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// === Modules ===

mod config;

#[cfg(feature = "sensors")]
mod sensors;

#[cfg(feature = "actuators")]
mod actuators;

#[cfg(all(feature = "linux", target_os = "linux"))]
mod linux;

mod shared;

// === CLI ===

#[derive(Parser)]
#[command(name = "rmcp-presence")]
#[command(about = "Unified MCP server for AI environmental awareness")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Open the config file in your editor to enable/disable tools
    Config,
}

// === Common Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EmptyParams {}

// === Server ===

#[derive(Debug)]
pub struct PresenceServer {
    pub tool_router: ToolRouter<Self>,
    #[cfg(feature = "sensors")]
    pub http_client: reqwest::Client,
}

impl Default for PresenceServer {
    fn default() -> Self {
        Self::new()
    }
}

impl PresenceServer {
    pub fn new() -> Self {
        let mut tool_router = Self::tool_router();

        // Load config and filter disabled tools
        let config = config::Config::load();
        let disabled_count = config.disabled.len();

        for tool_name in &config.disabled {
            if tool_router.has_route(tool_name) {
                tool_router.remove_route(tool_name);
                tracing::info!("Disabled tool: {}", tool_name);
            } else {
                tracing::warn!("Config disables unknown tool: {}", tool_name);
            }
        }

        if disabled_count > 0 {
            tracing::info!(
                "Loaded config: {} tools disabled, {} tools active",
                disabled_count,
                tool_router.map.len()
            );
        }

        Self {
            tool_router,
            #[cfg(feature = "sensors")]
            http_client: reqwest::Client::new(),
        }
    }
}

// Tool implementations
#[rmcp::tool_router]
impl PresenceServer {
    // ============================================================
    // SENSORS (Layer 1) - 30 tools
    // ============================================================

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get comprehensive context: datetime, user, environment, system state, battery - everything an AI needs to know about operating conditions")]
    pub async fn get_context(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::context::get_context().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get all connected peripherals: displays, USB devices, cameras, microphones, bluetooth - everything plugged in")]
    pub async fn get_peripherals(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::peripherals::get_peripherals().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get system overview: CPU usage, memory, disk space, uptime")]
    pub async fn get_system_info(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::sysinfo::get_system_info().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get top processes by CPU or memory usage")]
    pub async fn get_top_processes(
        &self,
        Parameters(params): Parameters<sensors::sysinfo::TopProcessesParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::sysinfo::get_top_processes(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Find processes by name (case-insensitive, partial match)")]
    pub async fn find_process(
        &self,
        Parameters(params): Parameters<sensors::sysinfo::FindProcessParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::sysinfo::find_process(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get detailed information about a specific process by PID")]
    pub async fn get_process_details(
        &self,
        Parameters(params): Parameters<sensors::sysinfo::ProcessIdParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::sysinfo::get_process_details(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "List all running processes (sorted by CPU usage)")]
    pub async fn list_processes(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::sysinfo::list_processes().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get network interface I/O statistics (bytes sent/received)")]
    pub async fn get_network_stats(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::sysinfo::get_network_stats().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get component temperatures (CPU, GPU, etc.)")]
    pub async fn get_component_temps(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::sysinfo::get_component_temps().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get detailed disk usage for all mounted filesystems")]
    pub async fn get_disk_info(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::sysinfo::get_disk_info().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get display/monitor information (connected displays, resolutions, physical sizes)")]
    pub async fn get_display_info(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::display::get_display_info().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get display info by name")]
    pub async fn get_display_by_name(
        &self,
        Parameters(params): Parameters<sensors::display::DisplayNameParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::display::get_display_by_name(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get display info at specific screen coordinates (useful for determining which monitor contains a point)")]
    pub async fn get_display_at_point(
        &self,
        Parameters(params): Parameters<sensors::display::DisplayPointParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::display::get_display_at_point(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get user idle time (how long since last keyboard/mouse input)")]
    pub async fn get_idle_time(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::idle::get_idle_time().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Check if user has been idle longer than specified seconds")]
    pub async fn is_idle_for(
        &self,
        Parameters(params): Parameters<sensors::idle::IdleThresholdParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::idle::is_idle_for(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "List all network interfaces with their IP addresses and MAC addresses")]
    pub async fn get_interfaces(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::network::get_interfaces().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get public IP address and geolocation info (city, region, country, ISP, timezone)")]
    pub async fn get_public_ip(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::network::get_public_ip().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Check if the system has internet connectivity (TCP connect to 1.1.1.1:53)")]
    pub async fn is_online(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::network::is_online().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Resolve a hostname to IP addresses via DNS")]
    pub async fn dns_lookup(
        &self,
        Parameters(params): Parameters<sensors::network::DnsLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::network::dns_lookup(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get comprehensive network status: online check, public IP, location, interfaces, and traffic stats - all in one call")]
    pub async fn get_network_info(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::network::get_network_info().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "List all connected USB devices with vendor/product info")]
    pub async fn get_usb_devices(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::usb::get_usb_devices().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get battery/power status (charge level, charging state, time remaining)")]
    pub async fn get_battery_status(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::battery::get_battery_status().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Scan for nearby Bluetooth Low Energy (BLE) devices")]
    pub async fn scan_ble_devices(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::bluetooth::scan_ble_devices().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get git repository status (branch, uncommitted changes, last commit)")]
    pub async fn get_status(
        &self,
        Parameters(params): Parameters<sensors::git::RepoPathParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::git::get_status(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get recent git commits (last 10)")]
    pub async fn get_log(
        &self,
        Parameters(params): Parameters<sensors::git::RepoPathParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::git::get_log(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "List all branches (local and remote)")]
    pub async fn get_branches(
        &self,
        Parameters(params): Parameters<sensors::git::RepoPathParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::git::get_branches(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get current branch name")]
    pub async fn get_current_branch(
        &self,
        Parameters(params): Parameters<sensors::git::RepoPathParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::git::get_current_branch(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "List all remotes with their URLs")]
    pub async fn get_remotes(
        &self,
        Parameters(params): Parameters<sensors::git::RepoPathParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::git::get_remotes(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "List all tags")]
    pub async fn get_tags(
        &self,
        Parameters(params): Parameters<sensors::git::RepoPathParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::git::get_tags(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "List stashed changes")]
    pub async fn get_stash_list(
        &self,
        Parameters(params): Parameters<sensors::git::RepoPathParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::git::get_stash_list(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get summary of uncommitted changes (file counts)")]
    pub async fn get_diff_summary(
        &self,
        Parameters(params): Parameters<sensors::git::RepoPathParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::git::get_diff_summary(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get comprehensive git info: branch, tracking, last commit, working tree status, remotes, stash count - all in one call")]
    pub async fn get_git_info(
        &self,
        Parameters(params): Parameters<sensors::git::RepoPathParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::git::get_git_info(params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get logged in users")]
    pub async fn get_users(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::sysinfo::get_users().await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get current weather conditions for a location")]
    pub async fn get_weather(
        &self,
        Parameters(params): Parameters<sensors::weather::LocationParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::weather::get_weather(&self.http_client, params).await
    }

    #[cfg(feature = "sensors")]
    #[rmcp::tool(description = "Get weather forecast for upcoming days")]
    pub async fn get_forecast(
        &self,
        Parameters(params): Parameters<sensors::weather::ForecastParams>,
    ) -> Result<CallToolResult, McpError> {
        sensors::weather::get_forecast(&self.http_client, params).await
    }

    // ============================================================
    // ACTUATORS (Layer 2) - 31 tools
    // ============================================================

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Read the current contents of the system clipboard")]
    pub async fn read_clipboard(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::clipboard::read_clipboard().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Write text to the system clipboard")]
    pub async fn write_clipboard(
        &self,
        Parameters(params): Parameters<actuators::clipboard::WriteClipboardParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::clipboard::write_clipboard(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Clear the system clipboard")]
    pub async fn clear_clipboard(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::clipboard::clear_clipboard().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Get current system volume as a percentage (0-100)")]
    pub async fn get_volume(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::audio::get_volume().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Set system volume to a percentage (0-100)")]
    pub async fn set_volume(
        &self,
        Parameters(params): Parameters<actuators::audio::SetVolumeParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::audio::set_volume(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Check if system audio is currently muted")]
    pub async fn get_mute(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::audio::get_mute().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Mute or unmute system audio")]
    pub async fn set_mute(
        &self,
        Parameters(params): Parameters<actuators::audio::SetMuteParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::audio::set_mute(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "List all available audio output devices")]
    pub async fn list_audio_devices(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::audio::list_audio_devices().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Move a file or directory to the system trash/recycle bin")]
    pub async fn trash_file(
        &self,
        Parameters(params): Parameters<actuators::trash::TrashFileParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::trash::trash_file(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Move multiple files or directories to the system trash/recycle bin")]
    pub async fn trash_files(
        &self,
        Parameters(params): Parameters<actuators::trash::TrashFilesParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::trash::trash_files(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "List items currently in the system trash (Linux/Windows only)")]
    pub async fn list_trash(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::trash::list_trash().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Restore a file from trash to its original location (Linux/Windows only)")]
    pub async fn restore_from_trash(
        &self,
        Parameters(params): Parameters<actuators::trash::RestoreParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::trash::restore_from_trash(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Permanently delete all items in the trash (Linux/Windows only). This cannot be undone!")]
    pub async fn empty_trash(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::trash::empty_trash().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Open a file, folder, or URL with the system default application")]
    pub async fn open_path(
        &self,
        Parameters(params): Parameters<actuators::open::OpenPathParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::open::open_path(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Open a file, folder, or URL with a specific application")]
    pub async fn open_with(
        &self,
        Parameters(params): Parameters<actuators::open::OpenWithParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::open::open_with(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "List all available monitors/displays")]
    pub async fn list_monitors(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::screenshot::list_monitors().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Capture a screenshot of a monitor. Returns base64 PNG image.")]
    pub async fn capture_monitor(
        &self,
        Parameters(params): Parameters<actuators::screenshot::CaptureMonitorParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::screenshot::capture_monitor(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "List all visible windows")]
    pub async fn list_windows(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::screenshot::list_windows().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Capture a screenshot of a specific window by title. Returns base64 PNG image.")]
    pub async fn capture_window(
        &self,
        Parameters(params): Parameters<actuators::screenshot::CaptureWindowParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::screenshot::capture_window(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Capture a specific region of the screen. Returns base64 PNG image.")]
    pub async fn capture_region(
        &self,
        Parameters(params): Parameters<actuators::screenshot::CaptureRegionParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::screenshot::capture_region(params).await
    }

    // === Camera Tools ===

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "List all available cameras/webcams on the system")]
    pub async fn list_cameras(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::camera::list_cameras().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Capture a photo from a camera. Returns base64 encoded JPEG.")]
    pub async fn capture_camera(
        &self,
        Parameters(params): Parameters<actuators::camera::CaptureParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::camera::capture_camera(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Get detailed information about a specific camera")]
    pub async fn get_camera_info(
        &self,
        Parameters(params): Parameters<actuators::camera::CameraIndexParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::camera::get_camera_info(params).await
    }

    // === Microphone Tools ===

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "List all available microphones/input devices on the system")]
    pub async fn list_microphones(
        &self,
        Parameters(_params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::microphone::list_microphones().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Get detailed information about a specific microphone")]
    pub async fn get_microphone_info(
        &self,
        Parameters(params): Parameters<actuators::microphone::MicrophoneIndexParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::microphone::get_microphone_info(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Record audio from a microphone. Returns base64 encoded WAV.")]
    pub async fn capture_audio(
        &self,
        Parameters(params): Parameters<actuators::microphone::CaptureParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::microphone::capture_audio(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Get current audio input level (0.0-1.0). Useful for voice activity detection.")]
    pub async fn get_input_level(
        &self,
        Parameters(params): Parameters<actuators::microphone::LevelParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::microphone::get_input_level(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "List all locally installed Ollama models")]
    pub async fn list_models(
        &self,
        Parameters(params): Parameters<actuators::ollama::HostParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::ollama::list_models(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "List models currently loaded in memory")]
    pub async fn list_running(
        &self,
        Parameters(params): Parameters<actuators::ollama::HostParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::ollama::list_running(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Show detailed information about a model")]
    pub async fn show_model(
        &self,
        Parameters(params): Parameters<actuators::ollama::ModelParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::ollama::show_model(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Pull (download) a model from Ollama registry")]
    pub async fn pull_model(
        &self,
        Parameters(params): Parameters<actuators::ollama::ModelParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::ollama::pull_model(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Delete a model from local storage")]
    pub async fn delete_model(
        &self,
        Parameters(params): Parameters<actuators::ollama::ModelParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::ollama::delete_model(params).await
    }

    // --- breakrs (6 tools) ---
    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Set a reminder/notification with natural language duration and message (e.g. '5m get coffee', '1h meeting', '30s tea ready')")]
    pub async fn set_reminder(
        &self,
        Parameters(params): Parameters<actuators::breakrs::SetReminderParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::breakrs::set_reminder(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "List all active/pending reminders")]
    pub async fn list_reminders(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        actuators::breakrs::list_reminders().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Remove/cancel a reminder by its ID (get IDs from list_reminders)")]
    pub async fn remove_reminder(
        &self,
        Parameters(params): Parameters<actuators::breakrs::RemoveReminderParams>,
    ) -> Result<CallToolResult, McpError> {
        actuators::breakrs::remove_reminder(params).await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Clear all active reminders (nuclear option)")]
    pub async fn clear_reminders(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        actuators::breakrs::clear_reminders().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Check if the breakrs daemon is running")]
    pub async fn daemon_status(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        actuators::breakrs::daemon_status().await
    }

    #[cfg(feature = "actuators")]
    #[rmcp::tool(description = "Show history of recently completed reminders")]
    pub async fn get_history(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        actuators::breakrs::get_history().await
    }

    // ============================================================
    // LINUX (Layer 3) - 83 tools
    // ============================================================

    // --- i3 (15 tools) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all i3 workspaces with their properties (number, name, visible, focused, urgent, output)")]
    pub async fn get_workspaces(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::i3::get_workspaces().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get the full i3 window tree (all containers, windows, and their layout)")]
    pub async fn get_tree(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::i3::get_tree().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Switch to a specific workspace by number or name")]
    pub async fn switch_workspace(&self, Parameters(params): Parameters<linux::i3::SwitchWorkspaceParams>) -> Result<CallToolResult, McpError> {
        linux::i3::switch_workspace(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Focus a window matching i3 criteria (e.g. [class=\"Firefox\"], [title=\"vim\"])")]
    pub async fn focus_window(&self, Parameters(params): Parameters<linux::i3::FocusWindowParams>) -> Result<CallToolResult, McpError> {
        linux::i3::focus_window(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Move the currently focused window to a specific workspace")]
    pub async fn move_to_workspace(&self, Parameters(params): Parameters<linux::i3::MoveToWorkspaceParams>) -> Result<CallToolResult, McpError> {
        linux::i3::move_to_workspace(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Execute any i3 command (escape hatch for advanced operations). See i3 user guide for command list.")]
    pub async fn run_command(&self, Parameters(params): Parameters<linux::i3::RunCommandParams>) -> Result<CallToolResult, McpError> {
        linux::i3::run_command(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Launch an application (e.g. 'firefox', 'kitty', 'emacs')")]
    pub async fn exec(&self, Parameters(params): Parameters<linux::i3::ExecParams>) -> Result<CallToolResult, McpError> {
        linux::i3::exec(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Kill (close) the currently focused window")]
    pub async fn kill(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::i3::kill().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Kill (close) a window matching i3 criteria (e.g. [class=\"Firefox\"], [title=\"vim\"])")]
    pub async fn kill_window(&self, Parameters(params): Parameters<linux::i3::KillWindowParams>) -> Result<CallToolResult, McpError> {
        linux::i3::kill_window(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Toggle fullscreen mode for the currently focused window")]
    pub async fn fullscreen(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::i3::fullscreen().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get all outputs/monitors with their properties (name, resolution, position, active status)")]
    pub async fn get_outputs(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::i3::get_outputs().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get all window marks (user-assigned labels for windows)")]
    pub async fn get_marks(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::i3::get_marks().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get all available binding modes (keyboard shortcut modes)")]
    pub async fn get_binding_modes(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::i3::get_binding_modes().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get i3 version information")]
    pub async fn get_version(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::i3::get_version().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get windows currently in the scratchpad")]
    pub async fn get_scratchpad(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::i3::get_scratchpad().await
    }

    // --- xdotool (12 tools) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Move mouse cursor to x,y coordinates on screen")]
    pub async fn move_mouse(&self, Parameters(params): Parameters<linux::xdotool::MoveMouseParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::move_mouse(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Click mouse button at current cursor position. Button: 1=left, 2=middle, 3=right")]
    pub async fn click(&self, Parameters(params): Parameters<linux::xdotool::ClickParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::click(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Move mouse to x,y coordinates and click. Button: 1=left, 2=middle, 3=right")]
    pub async fn click_at(&self, Parameters(params): Parameters<linux::xdotool::ClickAtParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::click_at(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Type text as keyboard input. Use for filling forms, search boxes, etc.")]
    pub async fn type_text(&self, Parameters(params): Parameters<linux::xdotool::TypeTextParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::type_text(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Press a key or combo. Examples: Return, Escape, ctrl+c, alt+Tab, super+1, ctrl+shift+t")]
    pub async fn key_press(&self, Parameters(params): Parameters<linux::xdotool::KeyPressParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::key_press(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Scroll mouse wheel. Direction: up, down, left, right")]
    pub async fn scroll(&self, Parameters(params): Parameters<linux::xdotool::ScrollParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::scroll(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get current mouse cursor position")]
    pub async fn get_mouse_position(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::get_mouse_position().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Double-click at current mouse position")]
    pub async fn double_click(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::double_click().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Search for windows by name, class, or pattern. Returns window IDs.")]
    pub async fn search_window(&self, Parameters(params): Parameters<linux::xdotool::SearchWindowParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::search_window(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get the currently focused/active window ID")]
    pub async fn get_active_window(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::get_active_window().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get window geometry (position and size) for a window ID")]
    pub async fn get_window_geometry(&self, Parameters(params): Parameters<linux::xdotool::WindowIdParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::get_window_geometry(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get the window title/name for a window ID")]
    pub async fn get_window_name(&self, Parameters(params): Parameters<linux::xdotool::WindowIdParams>) -> Result<CallToolResult, McpError> {
        linux::xdotool::get_window_name(params).await
    }

    // --- mpris (10 tools) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all running media players")]
    pub async fn list_players(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::list_players().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get currently playing track info (title, artist, album, status)")]
    pub async fn get_now_playing(&self, Parameters(params): Parameters<linux::mpris::PlayerParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::get_now_playing(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Start/resume playback")]
    pub async fn media_play(&self, Parameters(params): Parameters<linux::mpris::PlayerParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::media_play(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Pause playback")]
    pub async fn media_pause(&self, Parameters(params): Parameters<linux::mpris::PlayerParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::media_pause(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Toggle play/pause")]
    pub async fn media_play_pause(&self, Parameters(params): Parameters<linux::mpris::PlayerParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::media_play_pause(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Stop playback")]
    pub async fn media_stop(&self, Parameters(params): Parameters<linux::mpris::PlayerParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::media_stop(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Skip to next track")]
    pub async fn media_next(&self, Parameters(params): Parameters<linux::mpris::PlayerParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::media_next(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Go to previous track")]
    pub async fn media_previous(&self, Parameters(params): Parameters<linux::mpris::PlayerParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::media_previous(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get player volume (0.0 to 1.0)")]
    pub async fn get_player_volume(&self, Parameters(params): Parameters<linux::mpris::PlayerParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::get_player_volume(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Set player volume (0.0 to 1.0)")]
    pub async fn set_player_volume(&self, Parameters(params): Parameters<linux::mpris::SetVolumeParams>) -> Result<CallToolResult, McpError> {
        linux::mpris::set_player_volume(params).await
    }

    // --- systemd (7 tools) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List systemd units. Can filter by type (service, timer, socket, etc.) and state (active, inactive, failed).")]
    pub async fn list_units(&self, Parameters(params): Parameters<linux::systemd::ListParams>) -> Result<CallToolResult, McpError> {
        linux::systemd::list_units(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get detailed status of a systemd unit")]
    pub async fn get_unit_status(&self, Parameters(params): Parameters<linux::systemd::UnitParams>) -> Result<CallToolResult, McpError> {
        linux::systemd::get_unit_status(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Start a systemd unit")]
    pub async fn start_unit(&self, Parameters(params): Parameters<linux::systemd::UnitParams>) -> Result<CallToolResult, McpError> {
        linux::systemd::start_unit(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Stop a systemd unit")]
    pub async fn stop_unit(&self, Parameters(params): Parameters<linux::systemd::UnitParams>) -> Result<CallToolResult, McpError> {
        linux::systemd::stop_unit(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Restart a systemd unit")]
    pub async fn restart_unit(&self, Parameters(params): Parameters<linux::systemd::UnitParams>) -> Result<CallToolResult, McpError> {
        linux::systemd::restart_unit(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all failed systemd units")]
    pub async fn list_failed_units(&self, Parameters(params): Parameters<linux::systemd::FailedParams>) -> Result<CallToolResult, McpError> {
        linux::systemd::list_failed_units(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get recent journal logs for a systemd unit (via journalctl)")]
    pub async fn get_unit_logs(&self, Parameters(params): Parameters<linux::systemd::LogsParams>) -> Result<CallToolResult, McpError> {
        linux::systemd::get_unit_logs(params).await
    }

    // --- brightness (3 tools) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all brightness-controllable devices (displays/backlights)")]
    pub async fn list_brightness_devices(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::brightness::list_brightness_devices().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get current brightness level (0-100%) for a device")]
    pub async fn get_brightness(&self, Parameters(params): Parameters<linux::brightness::DeviceParams>) -> Result<CallToolResult, McpError> {
        linux::brightness::get_brightness(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Set brightness level (0-100%) for a device")]
    pub async fn set_brightness(&self, Parameters(params): Parameters<linux::brightness::SetBrightnessParams>) -> Result<CallToolResult, McpError> {
        linux::brightness::set_brightness(params).await
    }

    // --- bluer (9 tools) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all available Bluetooth adapters")]
    pub async fn list_adapters(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::bluer::list_adapters().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get detailed information about a Bluetooth adapter")]
    pub async fn get_adapter_info(&self, Parameters(params): Parameters<linux::bluer::AdapterParams>) -> Result<CallToolResult, McpError> {
        linux::bluer::get_adapter_info(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Scan for nearby Bluetooth devices (discovery). Returns devices found within the duration.")]
    pub async fn discover_devices(&self, Parameters(params): Parameters<linux::bluer::DiscoverParams>) -> Result<CallToolResult, McpError> {
        linux::bluer::discover_devices(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all known/paired Bluetooth devices")]
    pub async fn list_known_devices(&self, Parameters(params): Parameters<linux::bluer::AdapterParams>) -> Result<CallToolResult, McpError> {
        linux::bluer::list_known_devices(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get detailed information about a specific Bluetooth device")]
    pub async fn get_device_info(&self, Parameters(params): Parameters<linux::bluer::DeviceParams>) -> Result<CallToolResult, McpError> {
        linux::bluer::get_device_info(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Pair with a Bluetooth device by address")]
    pub async fn pair_device(&self, Parameters(params): Parameters<linux::bluer::DeviceParams>) -> Result<CallToolResult, McpError> {
        linux::bluer::pair_device(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Remove/unpair a Bluetooth device")]
    pub async fn remove_device(&self, Parameters(params): Parameters<linux::bluer::DeviceParams>) -> Result<CallToolResult, McpError> {
        linux::bluer::remove_device(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Connect to a paired Bluetooth device")]
    pub async fn connect_device(&self, Parameters(params): Parameters<linux::bluer::DeviceParams>) -> Result<CallToolResult, McpError> {
        linux::bluer::connect_device(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Disconnect from a Bluetooth device")]
    pub async fn disconnect_device(&self, Parameters(params): Parameters<linux::bluer::DeviceParams>) -> Result<CallToolResult, McpError> {
        linux::bluer::disconnect_device(params).await
    }

    // --- dbus (5 tools) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all available D-Bus service names on the bus")]
    pub async fn list_names(&self, Parameters(params): Parameters<linux::dbus::BusParams>) -> Result<CallToolResult, McpError> {
        linux::dbus::list_names(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Introspect a D-Bus object to see its interfaces, methods, and properties")]
    pub async fn introspect(&self, Parameters(params): Parameters<linux::dbus::IntrospectParams>) -> Result<CallToolResult, McpError> {
        linux::dbus::introspect(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Call a D-Bus method. Args should be a JSON array like [\"string\", 42, true]")]
    pub async fn call_method(&self, Parameters(params): Parameters<linux::dbus::MethodParams>) -> Result<CallToolResult, McpError> {
        linux::dbus::call_method(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get a D-Bus property value")]
    pub async fn get_property(&self, Parameters(params): Parameters<linux::dbus::PropertyParams>) -> Result<CallToolResult, McpError> {
        linux::dbus::get_property(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Set a D-Bus property value. Value should be JSON (string, number, boolean)")]
    pub async fn set_property(&self, Parameters(params): Parameters<linux::dbus::SetPropertyParams>) -> Result<CallToolResult, McpError> {
        linux::dbus::set_property(params).await
    }

    // --- logind (11 tools) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Suspend the system (sleep to RAM)")]
    pub async fn suspend(&self, Parameters(params): Parameters<linux::logind::InteractiveParams>) -> Result<CallToolResult, McpError> {
        linux::logind::suspend(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Hibernate the system (sleep to disk)")]
    pub async fn hibernate(&self, Parameters(params): Parameters<linux::logind::InteractiveParams>) -> Result<CallToolResult, McpError> {
        linux::logind::hibernate(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Power off the system")]
    pub async fn poweroff(&self, Parameters(params): Parameters<linux::logind::InteractiveParams>) -> Result<CallToolResult, McpError> {
        linux::logind::poweroff(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Reboot the system")]
    pub async fn reboot(&self, Parameters(params): Parameters<linux::logind::InteractiveParams>) -> Result<CallToolResult, McpError> {
        linux::logind::reboot(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Lock a user session's screen")]
    pub async fn lock_session(&self, Parameters(params): Parameters<linux::logind::SessionIdParams>) -> Result<CallToolResult, McpError> {
        linux::logind::lock_session(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all active user sessions")]
    pub async fn list_sessions(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::logind::list_sessions().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Check if the system can suspend")]
    pub async fn can_suspend(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::logind::can_suspend().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Check if the system can hibernate")]
    pub async fn can_hibernate(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::logind::can_hibernate().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Check if the system can power off")]
    pub async fn can_poweroff(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::logind::can_poweroff().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Check if the system can reboot")]
    pub async fn can_reboot(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::logind::can_reboot().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get all power capabilities at once")]
    pub async fn get_capabilities(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::logind::get_capabilities().await
    }

    // --- audio_status (1 tool) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get comprehensive audio status: volume, mute, default devices, now playing, apps using audio - all in one call")]
    pub async fn get_audio_status(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::audio_status::get_audio_status().await
    }

    // --- pulseaudio (11 tools) ---
    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all audio output devices (sinks)")]
    pub async fn list_sinks(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::list_sinks().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all audio input devices (sources/microphones)")]
    pub async fn list_sources(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::list_sources().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all applications currently playing audio (sink inputs)")]
    pub async fn list_sink_inputs(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::list_sink_inputs().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "List all applications currently recording audio (source outputs)")]
    pub async fn list_source_outputs(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::list_source_outputs().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get the default audio output device")]
    pub async fn get_default_sink(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::get_default_sink().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Get the default audio input device")]
    pub async fn get_default_source(&self, Parameters(_params): Parameters<EmptyParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::get_default_source().await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Set the default audio output device by name")]
    pub async fn set_default_sink(&self, Parameters(params): Parameters<linux::pulseaudio::NameParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::set_default_sink(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Set the default audio input device by name")]
    pub async fn set_default_source(&self, Parameters(params): Parameters<linux::pulseaudio::NameParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::set_default_source(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Adjust volume for a specific application (sink input). Use positive delta to increase, negative to decrease.")]
    pub async fn set_sink_input_volume(&self, Parameters(params): Parameters<linux::pulseaudio::VolumeParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::set_sink_input_volume(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Mute or unmute a specific application (sink input)")]
    pub async fn set_sink_input_mute(&self, Parameters(params): Parameters<linux::pulseaudio::MuteParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::set_sink_input_mute(params).await
    }

    #[cfg(all(feature = "linux", target_os = "linux"))]
    #[rmcp::tool(description = "Move an application's audio to a different output device (sink)")]
    pub async fn move_sink_input(&self, Parameters(params): Parameters<linux::pulseaudio::MoveAppParams>) -> Result<CallToolResult, McpError> {
        linux::pulseaudio::move_sink_input(params).await
    }
}

#[rmcp::tool_handler]
impl ServerHandler for PresenceServer {
    fn get_info(&self) -> ServerInfo {
        let mut description = String::from("rmcp-presence: Unified AI environmental awareness.\n");

        #[cfg(feature = "sensors")]
        description.push_str("- sensors: system, display, idle, network, usb, battery, bluetooth, git, weather\n");

        #[cfg(feature = "actuators")]
        description.push_str("- actuators: clipboard, audio, trash, open, screenshot, ollama, camera, microphone\n");

        #[cfg(all(feature = "linux", target_os = "linux"))]
        description.push_str("- linux: i3, xdotool, mpris, systemd, brightness, bluer, dbus, logind, pulseaudio\n");

        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(description),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config) => {
            run_config_command()?;
        }
        None => {
            run_server().await?;
        }
    }

    Ok(())
}

/// Open config file in user's editor
fn run_config_command() -> anyhow::Result<()> {
    let config_path = config::Config::path()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    // Create config dir if needed
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create config file from template if it doesn't exist
    if !config_path.exists() {
        let template = include_str!("../tools.toml.example");
        std::fs::write(&config_path, template)?;
        println!("Created config file: {}", config_path.display());
    }

    // Get editor from environment or use defaults
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            #[cfg(target_os = "windows")]
            { "notepad".to_string() }
            #[cfg(not(target_os = "windows"))]
            { "nano".to_string() }
        });

    println!("Opening {} with {}", config_path.display(), editor);

    // Open editor
    std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;

    Ok(())
}

/// Run the MCP server
async fn run_server() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting rmcp-presence server");

    let server = PresenceServer::new();
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    tracing::info!("rmcp-presence server stopped");
    Ok(())
}
