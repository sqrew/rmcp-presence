//! Configuration module for runtime tool filtering
//!
//! Reads/writes tool configuration from ~/.config/rmcp-presence/tools.toml

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// List of disabled tool names (all others are enabled)
    #[serde(default)]
    pub disabled: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            disabled: Vec::new(),
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("rmcp-presence").join("tools.toml"))
    }

    /// Load config from file, or return default if not found
    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            tracing::warn!("Could not determine config directory, using defaults");
            return Self::default();
        };

        if !path.exists() {
            tracing::info!("No config file found at {:?}, using defaults (all tools enabled)", path);
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => {
                    tracing::info!("Loaded config from {:?}", path);
                    config
                }
                Err(e) => {
                    tracing::error!("Failed to parse config file: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                tracing::error!("Failed to read config file: {}", e);
                Self::default()
            }
        }
    }

    /// Save config to file
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        tracing::info!("Saved config to {:?}", path);
        Ok(())
    }

    /// Check if a tool is enabled
    pub fn is_enabled(&self, tool_name: &str) -> bool {
        !self.disabled.contains(&tool_name.to_string())
    }

    /// Get set of disabled tools for fast lookup
    pub fn disabled_set(&self) -> HashSet<String> {
        self.disabled.iter().cloned().collect()
    }

    /// Enable a tool (remove from disabled list)
    pub fn enable(&mut self, tool_name: &str) {
        self.disabled.retain(|t| t != tool_name);
    }

    /// Disable a tool (add to disabled list)
    pub fn disable(&mut self, tool_name: &str) {
        if !self.disabled.contains(&tool_name.to_string()) {
            self.disabled.push(tool_name.to_string());
        }
    }
}

/// Get list of all available tool names (compile-time known)
pub fn all_tool_names() -> Vec<&'static str> {
    let mut tools = Vec::new();

    // === SENSORS ===
    #[cfg(feature = "sensors")]
    {
        tools.extend([
            "get_context",
            "get_peripherals",
            "get_system_info",
            "get_top_processes",
            "find_process",
            "get_process_details",
            "list_processes",
            "get_network_stats",
            "get_component_temps",
            "get_disk_info",
            "get_display_info",
            "get_display_by_name",
            "get_display_at_point",
            "get_idle_time",
            "is_idle_for",
            "get_interfaces",
            "get_public_ip",
            "is_online",
            "dns_lookup",
            "get_network_info",
            "get_usb_devices",
            "get_battery_status",
            "scan_ble_devices",
            "get_status",
            "get_log",
            "get_branches",
            "get_current_branch",
            "get_remotes",
            "get_tags",
            "get_stash_list",
            "get_diff_summary",
            "get_users",
            "get_weather",
            "get_forecast",
        ]);
    }

    // === ACTUATORS ===
    #[cfg(feature = "actuators")]
    {
        tools.extend([
            "read_clipboard",
            "write_clipboard",
            "clear_clipboard",
            "get_volume",
            "set_volume",
            "get_mute",
            "set_mute",
            "list_audio_devices",
            "trash_file",
            "trash_files",
            "list_trash",
            "restore_from_trash",
            "empty_trash",
            "open_path",
            "open_with",
            "list_monitors",
            "capture_monitor",
            "list_windows",
            "capture_window",
            "capture_region",
            "list_cameras",
            "capture_camera",
            "get_camera_info",
            "list_microphones",
            "get_microphone_info",
            "capture_audio",
            "get_input_level",
            "list_models",
            "list_running",
            "show_model",
            "pull_model",
            "delete_model",
            "set_reminder",
            "list_reminders",
            "remove_reminder",
            "clear_reminders",
            "daemon_status",
            "get_history",
        ]);
    }

    // === LINUX ===
    #[cfg(all(feature = "linux", target_os = "linux"))]
    {
        tools.extend([
            // i3
            "get_workspaces",
            "get_tree",
            "switch_workspace",
            "focus_window",
            "move_to_workspace",
            "run_command",
            "exec",
            "kill",
            "kill_window",
            "fullscreen",
            "get_outputs",
            "get_marks",
            "get_binding_modes",
            "get_version",
            "get_scratchpad",
            // xdotool
            "move_mouse",
            "click",
            "click_at",
            "type_text",
            "key_press",
            "scroll",
            "get_mouse_position",
            "double_click",
            "search_window",
            "get_active_window",
            "get_window_geometry",
            "get_window_name",
            // mpris
            "list_players",
            "get_now_playing",
            "media_play",
            "media_pause",
            "media_play_pause",
            "media_stop",
            "media_next",
            "media_previous",
            "get_player_volume",
            "set_player_volume",
            // systemd
            "list_units",
            "get_unit_status",
            "start_unit",
            "stop_unit",
            "restart_unit",
            "list_failed_units",
            "get_unit_logs",
            // brightness
            "list_brightness_devices",
            "get_brightness",
            "set_brightness",
            // bluer
            "list_adapters",
            "get_adapter_info",
            "discover_devices",
            "list_known_devices",
            "get_device_info",
            "pair_device",
            "remove_device",
            "connect_device",
            "disconnect_device",
            // dbus
            "list_names",
            "introspect",
            "call_method",
            "get_property",
            "set_property",
            // logind
            "suspend",
            "hibernate",
            "poweroff",
            "reboot",
            "lock_session",
            "list_sessions",
            "can_suspend",
            "can_hibernate",
            "can_poweroff",
            "can_reboot",
            "get_capabilities",
            // pulseaudio
            "list_sinks",
            "list_sources",
            "list_sink_inputs",
            "list_source_outputs",
            "get_default_sink",
            "get_default_source",
            "set_default_sink",
            "set_default_source",
            "set_sink_input_volume",
            "set_sink_input_mute",
            "move_sink_input",
        ]);
    }

    tools
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.disabled.is_empty());
        assert!(config.is_enabled("get_system_info"));
    }

    #[test]
    fn test_enable_disable() {
        let mut config = Config::default();

        config.disable("capture_audio");
        assert!(!config.is_enabled("capture_audio"));
        assert!(config.is_enabled("get_system_info"));

        config.enable("capture_audio");
        assert!(config.is_enabled("capture_audio"));
    }
}
