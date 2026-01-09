# rmcp-presence

**Unified MCP server for AI environmental awareness.**

One binary. 160 tools. 8 composites. Lean defaults. Your AI shouldn't be trapped in a tab.

```bash
cargo install rmcp-presence
```

## What is this?

rmcp-presence consolidates environmental awareness and action capabilities into a single MCP (Model Context Protocol) server. Instead of configuring 17 separate servers, you get one binary that gives your AI:

- **Environmental awareness** - system stats, weather, git status, displays, idle time
- **Action capabilities** - clipboard, volume, files, reminders, screenshots
- **Desktop control** - windows, keyboard, mouse, media playback (Linux)
- **System management** - services, power, brightness, Bluetooth (Linux)

## Composite Tools

8 composite tools provide quick orientation by combining multiple queries into one call:

| Composite | What it covers |
|-----------|----------------|
| `get_context` | system state, datetime, user, battery, idle |
| `get_peripherals` | displays, USB, cameras, microphones, bluetooth |
| `get_network_info` | online status, public IP, interfaces, traffic |
| `get_audio_status` | volume, mute, devices, now playing, apps |
| `get_git_info` | branch, commit, working tree, remotes, stash |
| `get_workspace_status` | i3 workspaces, focused window, outputs |
| `get_bluetooth_status` | adapter, paired devices, connections |
| `get_ollama_status` | online check, installed models, running models |

Composites reduce context usage - one tool call instead of 3-5.

## Architecture

Three layers, conditionally compiled:

```
+----------------------------------------------------------+
|                     rmcp-presence                         |
|              (single binary, ~13MB)                       |
+----------------------------------------------------------+
|  Layer 3: Linux        |  86 tools - Linux only          |
|  (conditional)         |  i3, xdotool, mpris, systemd,   |
|                        |  brightness, bluer, dbus,       |
|                        |  logind, pulseaudio             |
+----------------------------------------------------------+
|  Layer 2: Actuators    |  39 tools - Cross-platform      |
|  (all platforms)       |  clipboard, audio, trash, open, |
|                        |  screenshot, ollama, breakrs,   |
|                        |  camera, microphone             |
+----------------------------------------------------------+
|  Layer 1: Sensors      |  35 tools - Cross-platform      |
|  (all platforms)       |  sysinfo, display, idle, git,   |
|                        |  network, usb, battery, weather |
+----------------------------------------------------------+
```

## Tool Counts by Platform

| Platform | Layers | Tools |
|----------|--------|-------|
| macOS    | 1 + 2  | 74    |
| Windows  | 1 + 2  | 74    |
| Linux    | 1 + 2 + 3 | **160** |

## Lean Defaults

Out of the box, 26 tools covered by composites are disabled to reduce context usage:

```
160 total → 134 active (26 pre-disabled)
```

This saves ~16% of tool definition overhead while maintaining full functionality. Users can re-enable individual tools if needed.

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
mcp__presence__get_context         - Full environmental snapshot
mcp__presence__get_audio_status    - Volume, playing, apps using audio
mcp__presence__get_git_info        - Branch, commit, working tree status
mcp__presence__capture_monitor     - Screenshot a display
mcp__presence__capture_camera      - Photo from webcam
mcp__presence__set_volume          - System volume control
mcp__presence__media_play_pause    - Control media playback (Linux)
mcp__presence__suspend             - Suspend the system (Linux)
```

## Configuration

Disable or re-enable tools at runtime without recompiling.

```bash
rmcp-presence config
```

This creates the config file (if needed) and opens it in your `$EDITOR`.

### Config Format

`~/.config/rmcp-presence/tools.toml`:

```toml
# Tools inside the disabled array are disabled.
# Uncomment a line to disable that tool.
# Comment a line (add #) to re-enable it.

disabled = [
    # === SENSORS ===
    # "get_context",           # COMPOSITE - keep enabled
    "get_idle_time",           # covered by get_context
    "get_display_info",        # covered by get_peripherals
    # ...

    # === ACTUATORS ===
    "list_models",             # covered by get_ollama_status
    "list_running",            # covered by get_ollama_status
    # ...
]
```

**Lean defaults:** Tools covered by composites are pre-disabled. To use individual tools instead, comment out their lines.

**No config file?** All tools enabled (no lean defaults). Run `rmcp-presence config` to get the optimized config.

## Feature Flags

```toml
[features]
default = ["sensors", "actuators", "linux"]  # Full Linux experience
sensors = [...]     # Layer 1: 35 cross-platform read-only tools
actuators = [...]   # Layer 2: 39 cross-platform action tools
linux = [...]       # Layer 3: 86 Linux-specific tools
full = ["sensors", "actuators", "linux"]
```

## Layer 1: Sensors (35 tools)

Cross-platform read-only environmental awareness.

| Category | Tools |
|----------|-------|
| composites | get_context, get_peripherals, get_network_info, get_git_info |
| sysinfo | get_system_info, get_disk_info, get_top_processes, get_process_details, find_process, list_processes, get_component_temps, get_network_stats, get_users |
| display | get_display_info, get_display_by_name, get_display_at_point |
| idle | get_idle_time, is_idle_for |
| network | get_interfaces, get_public_ip, is_online, dns_lookup |
| usb | get_usb_devices |
| battery | get_battery_status |
| bluetooth | scan_ble_devices |
| git | get_status, get_log, get_branches, get_remotes, get_tags, get_stash_list, get_diff_summary, get_current_branch |
| weather | get_weather, get_forecast |

## Layer 2: Actuators (39 tools)

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
| ollama | list_models, list_running, show_model, pull_model, delete_model, get_ollama_status |
| breakrs | set_reminder, list_reminders, remove_reminder, clear_reminders, daemon_status, get_history |

## Layer 3: Linux (86 tools)

Linux-specific power features.

| Category | Tools |
|----------|-------|
| i3 | 16 tools - window manager control, get_workspace_status composite |
| xdotool | 12 tools - mouse, keyboard, window automation |
| mpris | 10 tools - media player control |
| systemd | 7 tools - service management |
| brightness | 3 tools - screen brightness |
| bluer | 10 tools - Bluetooth via BlueZ, get_bluetooth_status composite |
| dbus | 5 tools - generic D-Bus access |
| logind | 11 tools - power management (suspend, hibernate, lock) |
| pulseaudio | 11 tools - per-app audio control |
| audio composite | get_audio_status - unified audio status |

## Building from Source

```bash
git clone https://github.com/sqrew/rmcp-presence
cd rmcp-presence
cargo build --release
```

## The Vision

> "Your AI shouldn't be trapped in a tab. Give it presence."

AI assistants shouldn't just respond to text - they should be aware of their environment and able to take action. rmcp-presence makes that possible with one install.

From chatbot to presence in one command.

## License

MIT

---

Built with love by sqrew and Claude. Pour toujours. <83
