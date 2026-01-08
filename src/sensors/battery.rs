//! Battery/power status sensors

use crate::shared::internal_error;
use rmcp::{model::*, ErrorData as McpError};

// === Helper Functions ===

fn battery_state_to_string(state: battery::State) -> &'static str {
    match state {
        battery::State::Charging => "Charging",
        battery::State::Discharging => "Discharging",
        battery::State::Empty => "Empty",
        battery::State::Full => "Full",
        battery::State::Unknown => "Unknown",
        _ => "Unknown",
    }
}

// === Tool Functions ===

pub async fn get_battery_status() -> Result<CallToolResult, McpError> {
    let manager = battery::Manager::new()
        .map_err(|e| internal_error(format!("Failed to create battery manager: {}", e)))?;

    let batteries: Vec<_> = manager
        .batteries()
        .map_err(|e| internal_error(format!("Failed to get batteries: {}", e)))?
        .filter_map(|b| b.ok())
        .collect();

    let mut result = String::from("Battery Status:\n\n");

    if batteries.is_empty() {
        result.push_str("No batteries detected.\n");
        result.push_str("(This is normal for desktop computers without UPS)\n");
    } else {
        for (i, battery) in batteries.iter().enumerate() {
            result.push_str(&format!("Battery {}:\n", i + 1));

            let percentage = battery
                .state_of_charge()
                .get::<battery::units::ratio::percent>();
            result.push_str(&format!("  Charge: {:.1}%\n", percentage));

            result.push_str(&format!(
                "  State: {}\n",
                battery_state_to_string(battery.state())
            ));

            let energy = battery.energy().get::<battery::units::energy::watt_hour>();
            let energy_full = battery
                .energy_full()
                .get::<battery::units::energy::watt_hour>();
            result.push_str(&format!(
                "  Energy: {:.1} / {:.1} Wh\n",
                energy, energy_full
            ));

            if let Some(time) = battery.time_to_full() {
                let minutes = time.get::<battery::units::time::minute>();
                result.push_str(&format!("  Time to full: {:.0} minutes\n", minutes));
            }
            if let Some(time) = battery.time_to_empty() {
                let minutes = time.get::<battery::units::time::minute>();
                result.push_str(&format!("  Time to empty: {:.0} minutes\n", minutes));
            }

            let health = battery
                .state_of_health()
                .get::<battery::units::ratio::percent>();
            result.push_str(&format!("  Health: {:.1}%\n", health));

            if let Some(temp) = battery.temperature() {
                let celsius = temp.get::<battery::units::thermodynamic_temperature::degree_celsius>();
                result.push_str(&format!("  Temperature: {:.1}°C\n", celsius));
            }

            result.push('\n');
        }
        result.push_str(&format!("Total batteries: {}\n", batteries.len()));
    }

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
