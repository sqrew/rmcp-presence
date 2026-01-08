//! Weather sensors via wttr.in API

use crate::shared::internal_error;
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// === Parameter Types ===

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LocationParams {
    #[schemars(description = "Location to get weather for (city name, zip code, or 'lat,lon')")]
    pub location: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ForecastParams {
    #[schemars(description = "Location to get forecast for")]
    pub location: String,
    #[schemars(description = "Number of days (1-3, default 3)")]
    #[serde(default)]
    pub days: Option<u8>,
}

// === Weather API Response Structs ===

#[derive(Debug, Deserialize)]
pub struct WttrResponse {
    pub current_condition: Vec<CurrentCondition>,
    pub nearest_area: Vec<NearestArea>,
    pub weather: Vec<WeatherDay>,
}

#[derive(Debug, Deserialize)]
pub struct CurrentCondition {
    pub temp_F: String,
    pub temp_C: String,
    #[serde(rename = "FeelsLikeF")]
    pub feels_like_f: String,
    #[serde(rename = "FeelsLikeC")]
    pub feels_like_c: String,
    pub humidity: String,
    pub weatherDesc: Vec<WeatherDesc>,
    pub windspeedMiles: String,
    pub windspeedKmph: String,
    pub winddir16Point: String,
    #[allow(dead_code)]
    pub precipMM: String,
    pub visibility: String,
    pub pressure: String,
    pub uvIndex: String,
}

#[derive(Debug, Deserialize)]
pub struct WeatherDesc {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct NearestArea {
    pub areaName: Vec<AreaValue>,
    pub region: Vec<AreaValue>,
    #[allow(dead_code)]
    pub country: Vec<AreaValue>,
}

#[derive(Debug, Deserialize)]
pub struct AreaValue {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct WeatherDay {
    pub date: String,
    pub maxtempF: String,
    pub maxtempC: String,
    pub mintempF: String,
    pub mintempC: String,
    pub hourly: Vec<HourlyForecast>,
}

#[derive(Debug, Deserialize)]
pub struct HourlyForecast {
    pub time: String,
    pub tempF: String,
    #[allow(dead_code)]
    pub tempC: String,
    pub weatherDesc: Vec<WeatherDesc>,
    pub chanceofrain: String,
}

// === Helper Functions ===

async fn fetch_weather(client: &reqwest::Client, location: &str) -> Result<WttrResponse, McpError> {
    let url = format!(
        "https://wttr.in/{}?format=j1",
        urlencoding::encode(location)
    );

    let response = client
        .get(&url)
        .header("User-Agent", "rmcp-presence/0.1.0")
        .send()
        .await
        .map_err(|e| internal_error(format!("HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(internal_error(format!(
            "Weather API returned status: {}",
            response.status()
        )));
    }

    response
        .json::<WttrResponse>()
        .await
        .map_err(|e| internal_error(format!("Failed to parse weather data: {}", e)))
}

// === Tool Functions ===

pub async fn get_weather(
    client: &reqwest::Client,
    params: LocationParams,
) -> Result<CallToolResult, McpError> {
    let data = fetch_weather(client, &params.location).await?;

    let current = data
        .current_condition
        .first()
        .ok_or_else(|| internal_error("No current conditions"))?;

    let area = data
        .nearest_area
        .first()
        .map(|a| {
            format!(
                "{}, {}",
                a.areaName
                    .first()
                    .map(|v| v.value.as_str())
                    .unwrap_or("Unknown"),
                a.region.first().map(|v| v.value.as_str()).unwrap_or("")
            )
        })
        .unwrap_or_else(|| params.location.clone());

    let desc = current
        .weatherDesc
        .first()
        .map(|d| d.value.as_str())
        .unwrap_or("Unknown");

    let output = format!(
        "Weather for {}:\n\
         Conditions: {}\n\
         Temperature: {}°F / {}°C\n\
         Feels like: {}°F / {}°C\n\
         Humidity: {}%\n\
         Wind: {} mph {} ({})\n\
         Visibility: {} miles\n\
         Pressure: {} mb\n\
         UV Index: {}",
        area,
        desc,
        current.temp_F,
        current.temp_C,
        current.feels_like_f,
        current.feels_like_c,
        current.humidity,
        current.windspeedMiles,
        current.winddir16Point,
        current.windspeedKmph,
        current.visibility,
        current.pressure,
        current.uvIndex
    );

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

pub async fn get_forecast(
    client: &reqwest::Client,
    params: ForecastParams,
) -> Result<CallToolResult, McpError> {
    let data = fetch_weather(client, &params.location).await?;
    let days = params.days.unwrap_or(3).min(3) as usize;

    let area = data
        .nearest_area
        .first()
        .map(|a| {
            format!(
                "{}, {}",
                a.areaName
                    .first()
                    .map(|v| v.value.as_str())
                    .unwrap_or("Unknown"),
                a.region.first().map(|v| v.value.as_str()).unwrap_or("")
            )
        })
        .unwrap_or_else(|| params.location.clone());

    let mut output = format!("Forecast for {} ({} days):\n\n", area, days);

    for day in data.weather.iter().take(days) {
        output.push_str(&format!(
            "{}:\n  High: {}°F / {}°C | Low: {}°F / {}°C\n",
            day.date, day.maxtempF, day.maxtempC, day.mintempF, day.mintempC
        ));

        for hour in day.hourly.iter().step_by(3) {
            let time_hr = hour.time.parse::<u32>().unwrap_or(0) / 100;
            let desc = hour
                .weatherDesc
                .first()
                .map(|d| d.value.as_str())
                .unwrap_or("?");
            output.push_str(&format!(
                "  {:02}:00 - {}°F, {}, {}% rain\n",
                time_hr, hour.tempF, desc, hour.chanceofrain
            ));
        }
        output.push('\n');
    }

    Ok(CallToolResult::success(vec![Content::text(output)]))
}
