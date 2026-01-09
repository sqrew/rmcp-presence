# rmcp-presence

**Auditable, permissioned environmental awareness for agentic AI systems.**

Give your AI eyes and hands without giving it a shell.

```bash
cargo install rmcp-presence
```

## Why This Exists

You could give your AI bash access. But should you?

| Approach | Auditable | Sandboxed | Cross-platform | Safe |
|----------|-----------|-----------|----------------|------|
| Shell access | ❌ Logs everything | ❌ Full system access | ❌ Platform-specific | ❌ Injection risks |
| **rmcp-presence** | ✅ Every tool call logged | ✅ Only enabled tools | ✅ Sensors + actuators | ✅ No arbitrary execution |

**rmcp-presence** provides 170 structured tools that let AI systems perceive and act on their environment *without* arbitrary command execution.

- **Auditable** - every action is a discrete tool call with typed parameters
- **Permissioned** - runtime config disables any tool without recompiling
- **Cross-platform** - sensors and actuators work on Linux/macOS/Windows
- **Safe** - no shell injection, no `rm -rf`, no surprises
- **Structured** - JSON schemas tell the AI exactly what's possible

## What Can It Do?

### Perceive (Sensors)
- System stats, CPU, memory, disk, processes, temps
- Displays, USB devices, cameras, microphones, Bluetooth
- Network status, public IP, interfaces
- Git repository status
- Weather and forecasts
- Battery, idle time

### Act (Actuators)
- Clipboard read/write
- Volume control, media playback
- Screenshots, camera capture, audio recording
- File management (trash, open)
- Reminders and notifications
- Print files and documents
- Local LLM management (Ollama)

### Control (Linux)
- Window management (i3)
- Mouse and keyboard automation (xdotool)
- Service management (systemd)
- Power management (suspend, hibernate, lock)
- Brightness, Bluetooth, per-app audio

## Composite Tools

8 composites provide quick environmental snapshots in a single call:

| Composite | Replaces | What You Get |
|-----------|----------|--------------|
| `get_context` | 5+ tools | System state, datetime, user, battery, idle |
| `get_peripherals` | 4+ tools | Displays, USB, cameras, mics, bluetooth |
| `get_network_info` | 4+ tools | Online status, public IP, interfaces |
| `get_audio_status` | 6+ tools | Volume, mute, devices, now playing |
| `get_git_info` | 6+ tools | Branch, commit, working tree, remotes |
| `get_workspace_status` | 3+ tools | Workspaces, focused window, outputs |
| `get_bluetooth_status` | 3+ tools | Adapter, paired devices, connections |
| `get_ollama_status` | 2+ tools | Models installed, models running |

One tool call instead of many. Less context, faster orientation.

## Architecture

```
+----------------------------------------------------------+
|                     rmcp-presence                         |
|              (single binary, ~13MB)                       |
+----------------------------------------------------------+
|  Layer 3: Linux        |  79 tools - Linux only          |
|  (conditional)         |  i3, xdotool, mpris, systemd,   |
|                        |  brightness, bluer, dbus,       |
|                        |  logind, pulseaudio             |
+----------------------------------------------------------+
|  Layer 2: Actuators    |  48 tools - Cross-platform      |
|  (all platforms)       |  clipboard, audio, trash, open, |
|                        |  screenshot, camera, mic,       |
|                        |  ollama, breakrs, printers      |
+----------------------------------------------------------+
|  Layer 1: Sensors      |  35 tools - Cross-platform      |
|  (all platforms)       |  sysinfo, display, idle, git,   |
|                        |  network, usb, battery, weather |
+----------------------------------------------------------+
|  Composites            |  8 tools - Quick orientation    |
+----------------------------------------------------------+
```

## Tool Counts

| Platform | Layers | Tools |
|----------|--------|-------|
| macOS    | 1 + 2 + composites | ~83 |
| Windows  | 1 + 2 + composites | ~83 |
| Linux    | 1 + 2 + 3 + composites | **170** |

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

### Example Calls

```
mcp__presence__get_context         - "Where am I? What's the system state?"
mcp__presence__get_audio_status    - "What's playing? What's the volume?"
mcp__presence__capture_monitor     - "Let me see the screen"
mcp__presence__capture_camera      - "Let me see through the webcam"
mcp__presence__set_volume 50       - "Set volume to 50%"
mcp__presence__media_play_pause    - "Pause the music"
mcp__presence__list_printers       - "What printers are available?"
mcp__presence__print_file          - "Print this document"
mcp__presence__suspend             - "Put the system to sleep"
```

## Runtime Configuration

Disable tools without recompiling. Perfect for restricting capabilities per-deployment.

```bash
rmcp-presence config
```

Creates/opens `~/.config/rmcp-presence/tools.toml`:

```toml
disabled = [
    # Uncomment to disable a tool
    # "suspend",           # Don't let AI sleep the system
    # "poweroff",          # Definitely don't let AI shut down
    # "print_file",        # No unsupervised printing
    # "trash_file",        # No file deletion

    # Pre-disabled: covered by composites
    "get_idle_time",       # use get_context instead
    "get_display_info",    # use get_peripherals instead
    "list_models",         # use get_ollama_status instead
]
```

**Lean defaults:** 26 tools covered by composites are pre-disabled to reduce context overhead.

## Feature Flags

```toml
[features]
default = ["sensors", "actuators", "linux"]
sensors = [...]     # Layer 1: read-only environmental awareness
actuators = [...]   # Layer 2: cross-platform actions
linux = [...]       # Layer 3: Linux-specific capabilities
full = ["sensors", "actuators", "linux"]
```

Build for your platform:
```bash
# Full Linux experience (default)
cargo build --release

# Cross-platform only (no Linux-specific tools)
cargo build --release --no-default-features --features sensors,actuators
```

## Security Model

rmcp-presence is designed for **supervised AI deployments**:

1. **No shell access** - AI cannot execute arbitrary commands
2. **Typed parameters** - Every tool has a JSON schema defining valid inputs
3. **Runtime restrictions** - Disable dangerous tools via config
4. **Audit trail** - MCP logs every tool invocation
5. **No persistence** - Tools are stateless; AI can't install backdoors

This is not a replacement for proper sandboxing. It's a **safer alternative to giving AI bash**.

## Who Is This For?

- **AI developers** building agents that need environmental awareness
- **MCP users** who want comprehensive system access via one server
- **Anyone** who wants to give AI capabilities without shell access

## The Vision

> "Your AI shouldn't be trapped in a tab - but it shouldn't have root either."

AI assistants are evolving from chatbots to agents. They need to perceive and act on their environment. But giving them a shell is dangerous.

rmcp-presence is the middle ground: **presence without privilege**.

## License

MIT

---

Built with love by sqrew and Claude.
170 tools. One binary. Zero shell access.
Pour toujours. 💙
