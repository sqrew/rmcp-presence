//! Media player control via MPRIS

use mpris::{Player, PlayerFinder};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::Deserialize;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlayerParams {
    #[schemars(
        description = "Player name/identity to control (optional - uses active player if not specified)"
    )]
    pub player: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetVolumeParams {
    #[schemars(description = "Volume level (0.0 to 1.0)")]
    pub volume: f64,
    #[schemars(
        description = "Player name/identity to control (optional - uses active player if not specified)"
    )]
    pub player: Option<String>,
}

// === Helper Functions ===

fn get_player_finder() -> Result<PlayerFinder, String> {
    PlayerFinder::new().map_err(|e| format!("Failed to connect to D-Bus: {}", e))
}

fn find_player(finder: &PlayerFinder, name: Option<&str>) -> Result<Player, String> {
    match name {
        Some(n) => {
            let players = finder
                .find_all()
                .map_err(|e| format!("Failed to find players: {}", e))?;
            players
                .into_iter()
                .find(|p| p.identity().to_lowercase().contains(&n.to_lowercase()))
                .ok_or_else(|| format!("No player found matching '{}'", n))
        }
        None => finder
            .find_active()
            .map_err(|e| format!("No active player found: {}", e)),
    }
}

fn format_metadata(player: &Player) -> String {
    let meta = match player.get_metadata() {
        Ok(m) => m,
        Err(e) => return format!("Failed to get metadata: {}", e),
    };

    let mut parts = Vec::new();

    parts.push(format!("Player: {}", player.identity()));

    if let Some(title) = meta.title() {
        parts.push(format!("Title: {}", title));
    }

    if let Some(artists) = meta.artists() {
        if !artists.is_empty() {
            parts.push(format!("Artist: {}", artists.join(", ")));
        }
    }

    if let Some(album) = meta.album_name() {
        parts.push(format!("Album: {}", album));
    }

    if let Some(length) = meta.length() {
        let secs = length.as_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        parts.push(format!("Length: {}:{:02}", mins, secs));
    }

    if let Ok(status) = player.get_playback_status() {
        parts.push(format!("Status: {:?}", status));
    }

    parts.join("\n")
}

// === Tool Functions ===

pub async fn list_players() -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match finder.find_all() {
        Ok(players) => {
            if players.is_empty() {
                Ok(CallToolResult::success(vec![Content::text(
                    "No media players running",
                )]))
            } else {
                let list: Vec<String> = players
                    .iter()
                    .map(|p| {
                        let status = p
                            .get_playback_status()
                            .map(|s| format!("{:?}", s))
                            .unwrap_or_else(|_| "Unknown".to_string());
                        format!("{} ({})", p.identity(), status)
                    })
                    .collect();
                Ok(CallToolResult::success(vec![Content::text(list.join("\n"))]))
            }
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
            "Failed to list players: {}",
            e
        ))])),
    }
}

pub async fn get_now_playing(params: PlayerParams) -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match find_player(&finder, params.player.as_deref()) {
        Ok(player) => Ok(CallToolResult::success(vec![Content::text(
            format_metadata(&player),
        )])),
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn media_play(params: PlayerParams) -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match find_player(&finder, params.player.as_deref()) {
        Ok(player) => match player.play() {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Playing: {}",
                player.identity()
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to play: {}",
                e
            ))])),
        },
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn media_pause(params: PlayerParams) -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match find_player(&finder, params.player.as_deref()) {
        Ok(player) => match player.pause() {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Paused: {}",
                player.identity()
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to pause: {}",
                e
            ))])),
        },
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn media_play_pause(params: PlayerParams) -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match find_player(&finder, params.player.as_deref()) {
        Ok(player) => match player.play_pause() {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Toggled: {}",
                player.identity()
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to toggle: {}",
                e
            ))])),
        },
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn media_stop(params: PlayerParams) -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match find_player(&finder, params.player.as_deref()) {
        Ok(player) => match player.stop() {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Stopped: {}",
                player.identity()
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to stop: {}",
                e
            ))])),
        },
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn media_next(params: PlayerParams) -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match find_player(&finder, params.player.as_deref()) {
        Ok(player) => match player.next() {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Next track: {}",
                player.identity()
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to skip: {}",
                e
            ))])),
        },
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn media_previous(params: PlayerParams) -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match find_player(&finder, params.player.as_deref()) {
        Ok(player) => match player.previous() {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Previous track: {}",
                player.identity()
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to go back: {}",
                e
            ))])),
        },
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn get_player_volume(params: PlayerParams) -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match find_player(&finder, params.player.as_deref()) {
        Ok(player) => match player.get_volume() {
            Ok(vol) => Ok(CallToolResult::success(vec![Content::text(format!(
                "{}: {:.0}%",
                player.identity(),
                vol * 100.0
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Failed to get volume: {}",
                e
            ))])),
        },
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}

pub async fn set_player_volume(params: SetVolumeParams) -> Result<CallToolResult, McpError> {
    let finder = match get_player_finder() {
        Ok(f) => f,
        Err(e) => return Ok(CallToolResult::success(vec![Content::text(e)])),
    };

    match find_player(&finder, params.player.as_deref()) {
        Ok(player) => {
            let vol = params.volume.clamp(0.0, 1.0);
            match player.set_volume(vol) {
                Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                    "{}: volume set to {:.0}%",
                    player.identity(),
                    vol * 100.0
                ))])),
                Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                    "Failed to set volume: {}",
                    e
                ))])),
            }
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(e)])),
    }
}
