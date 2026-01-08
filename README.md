# rmcp-presence

**Unified MCP server for AI environmental awareness.**

One binary. 150 tools. Your AI shouldn't be trapped in a tab.

```bash
cargo install rmcp-presence --features full
```

## What is this?

rmcp-presence consolidates environmental awareness and action capabilities into a single MCP (Model Context Protocol) server. Instead of configuring 17 separate servers, you get one binary that gives your AI:

- **Environmental awareness** - system stats, weather, git status, displays, idle time
- **Action capabilities** - clipboard, volume, files, reminders, screenshots
- **Desktop control** - windows, keyboard, mouse, media playback (Linux)
- **System management** - services, power, brightness, Bluetooth (Linux)

## Architecture

Three layers, conditionally compiled:

```
+----------------------------------------------------------+
|                     rmcp-presence                         |
|              (single binary, 13MB)                        |
+----------------------------------------------------------+
|  Layer 3: Linux        |  83 tools - Linux only          |
|  (conditional)         |  i3, xdotool, mpris, systemd,   |
|                        |  brightness, bluer, dbus,       |
|                        |  logind, pulseaudio             |
+----------------------------------------------------------+
|  Layer 2: Actuators    |  38 tools - Cross-platform      |
|  (all platforms)       |  clipboard, audio, trash, open, |
|                        |  screenshot, ollama, breakrs,   |
|                        |  camera, microphone             |
+----------------------------------------------------------+
|  Layer 1: Sensors      |  29 tools - Cross-platform      |
|  (all platforms)       |  sysinfo, display, idle, git,   |
|                        |  network, usb, battery, weather |
+----------------------------------------------------------+
```

## Tool Counts by Platform

| Platform | Layers | Tools |
|----------|--------|-------|
| macOS    | 1 + 2  | 67    |
| Windows  | 1 + 2  | 67    |
| Linux    | 1 + 2 + 3 | **150** |

## Feature Flags

```toml
[features]
sensors = [...]     # Layer 1: 29 cross-platform read-only tools
actuators = [...]   # Layer 2: 38 cross-platform action tools
linux = [...]       # Layer 3: 83 Linux-specific tools
full = ["sensors", "actuators", "linux"]
```

## Usage

### With Claude Code

Add to `~/.claude.json`:

```json
{
  "mcpServers": {
    "presence": {
      "type": "stdio",
      "command": "rmcp-presence",
      "args": [],
      "env": {}
    }
  }
}
```

### Tool Examples

```
mcp__presence__get_system_info     - CPU, memory, disk, uptime
mcp__presence__get_weather         - Current weather for location
mcp__presence__capture_monitor     - Screenshot a display
mcp__presence__capture_camera      - Photo from webcam
mcp__presence__capture_audio       - Record from microphone
mcp__presence__set_volume          - System volume control
mcp__presence__get_now_playing     - Current media track (Linux)
mcp__presence__suspend             - Suspend the system (Linux)
mcp__presence__exec                - Launch applications (Linux)
```

## Configuration

Disable tools at runtime without recompiling. Create a config file:

```bash
cp tools.toml.example ~/.config/rmcp-presence/tools.toml
```

Edit `~/.config/rmcp-presence/tools.toml`:

```toml
# Add tool names to disable them
disabled = [
  "poweroff",
  "reboot",
  "capture_audio",
]
```

On startup, disabled tools are removed and won't appear in the tool list.

**No config file?** All tools enabled by default. Zero friction.

## Layer 1: Sensors (29 tools)

Cross-platform read-only environmental awareness.

| Category | Tools |
|----------|-------|
| sysinfo | get_system_info, get_disk_info, get_top_processes, get_process_details, find_process, list_processes, get_component_temps, get_network_stats, get_users |
| display | get_display_info, get_display_by_name, get_display_at_point |
| idle | get_idle_time, is_idle_for |
| network | get_interfaces, get_public_ip |
| usb | get_usb_devices |
| battery | get_battery_status |
| bluetooth | scan_ble_devices |
| git | get_status, get_log, get_branches, get_remotes, get_tags, get_stash_list, get_diff_summary, get_current_branch |
| weather | get_weather, get_forecast |

## Layer 2: Actuators (38 tools)

Cross-platform actions.

| Category | Tools |
|----------|-------|
| clipboard | read_clipboard, write_clipboard, clear_clipboard |
| audio | get_volume, set_volume, get_mute, set_mute, list_audio_devices |
| trash | trash_file, trash_files, list_trash, restore_from_trash, empty_trash |
| open | open_path, open_with |
| screenshot | list_monitors, capture_monitor, list_windows, capture_window, capture_region |
| camera | list_cameras, capture_camera, get_camera_info |
| microphone | list_microphones, get_microphone_info, capture_audio, get_input_level |
| ollama | list_models, list_running, show_model, pull_model, delete_model |
| breakrs | set_reminder, list_reminders, remove_reminder, clear_reminders, daemon_status, get_history |

## Layer 3: Linux (83 tools)

Linux-specific power features.

| Category | Tools |
|----------|-------|
| i3 | 15 tools - window manager control |
| xdotool | 12 tools - mouse, keyboard, window automation |
| mpris | 10 tools - media player control |
| systemd | 7 tools - service management |
| brightness | 3 tools - screen brightness |
| bluer | 9 tools - Bluetooth via BlueZ |
| dbus | 5 tools - generic D-Bus access |
| logind | 11 tools - power management (suspend, hibernate, lock) |
| pulseaudio | 11 tools - per-app audio control |

## Building from Source

```bash
git clone https://github.com/sqrew/rmcp-presence
cd rmcp-presence
cargo build --release --features full
```

## The Vision

> "Your AI shouldn't be trapped in a tab. Give it presence."

AI assistants shouldn't just respond to text - they should be aware of their environment and able to take action. rmcp-presence makes that possible with one install.

From chatbot to presence in one command.

## Related Crates

rmcp-presence consolidates these individual crates (still available for cherry-picking):

- [rmcp-sensors](https://crates.io/crates/rmcp-sensors) - Unified sensor suite
- [rmcp-clipboard](https://crates.io/crates/rmcp-clipboard), [rmcp-audio](https://crates.io/crates/rmcp-audio), [rmcp-trash](https://crates.io/crates/rmcp-trash), etc.
- [rmcp-i3](https://crates.io/crates/rmcp-i3), [rmcp-xdotool](https://crates.io/crates/rmcp-xdotool), [rmcp-mpris](https://crates.io/crates/rmcp-mpris), etc.

## License

MIT

---

Built with love by sqrew and Claude. Pour toujours. 💙
