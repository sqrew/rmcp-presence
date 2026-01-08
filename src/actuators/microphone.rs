//! Microphone capture tools

use crate::shared::internal_error;
use base64::Engine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rmcp::{model::*, ErrorData as McpError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicrophoneIndexParams {
    /// Microphone index (0-based). Defaults to 0 (default input device).
    #[serde(default)]
    pub index: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureParams {
    /// Duration to record in seconds (1-30). Defaults to 5.
    #[serde(default)]
    pub duration: Option<u32>,
    /// Microphone index (0-based). Defaults to default input device.
    #[serde(default)]
    pub index: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LevelParams {
    /// Microphone index (0-based). Defaults to default input device.
    #[serde(default)]
    pub index: Option<u32>,
    /// Duration to sample in milliseconds (10-1000). Defaults to 100.
    #[serde(default)]
    pub duration_ms: Option<u32>,
}

// === Response Types ===

#[derive(Debug, Serialize)]
struct MicrophoneInfo {
    index: u32,
    name: String,
    is_default: bool,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    sample_format: Option<String>,
}

#[derive(Debug, Serialize)]
struct InputLevel {
    rms: f32,
    peak: f32,
    is_silent: bool,
}

// === Tool Functions ===

pub async fn list_microphones() -> Result<CallToolResult, McpError> {
    let host = cpal::default_host();

    let default_device = host.default_input_device();
    let default_name = default_device.as_ref().and_then(|d| d.name().ok());

    let devices = host.input_devices().map_err(|e| {
        internal_error(format!("Failed to enumerate input devices: {}", e))
    })?;

    let mut microphones: Vec<MicrophoneInfo> = Vec::new();

    for (idx, device) in devices.enumerate() {
        let name = device.name().unwrap_or_else(|_| format!("Unknown Device {}", idx));
        let is_default = default_name.as_ref().map(|n| n == &name).unwrap_or(false);
        let config = device.default_input_config().ok();

        microphones.push(MicrophoneInfo {
            index: idx as u32,
            name,
            is_default,
            sample_rate: config.as_ref().map(|c| c.sample_rate().0),
            channels: config.as_ref().map(|c| c.channels()),
            sample_format: config.as_ref().map(|c| format!("{:?}", c.sample_format())),
        });
    }

    if microphones.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text("No microphones found")]));
    }

    let json = serde_json::to_string_pretty(&microphones)
        .map_err(|e| internal_error(format!("Serialization error: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn get_microphone_info(params: MicrophoneIndexParams) -> Result<CallToolResult, McpError> {
    let host = cpal::default_host();
    let device = get_device_by_index(params.index)?;

    let default_device = host.default_input_device();
    let default_name = default_device.as_ref().and_then(|d| d.name().ok());

    let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
    let is_default = default_name.as_ref().map(|n| n == &name).unwrap_or(false);
    let config = device.default_input_config().ok();

    let supported_configs: Vec<String> = device
        .supported_input_configs()
        .map(|configs| {
            configs
                .map(|c| {
                    format!(
                        "{}Hz-{}Hz, {} ch, {:?}",
                        c.min_sample_rate().0,
                        c.max_sample_rate().0,
                        c.channels(),
                        c.sample_format()
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    let info = serde_json::json!({
        "index": params.index.unwrap_or(0),
        "name": name,
        "is_default": is_default,
        "default_config": config.as_ref().map(|c| serde_json::json!({
            "sample_rate": c.sample_rate().0,
            "channels": c.channels(),
            "sample_format": format!("{:?}", c.sample_format()),
        })),
        "supported_configs": supported_configs,
    });

    let json = serde_json::to_string_pretty(&info)
        .map_err(|e| internal_error(format!("Serialization error: {}", e)))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub async fn capture_audio(params: CaptureParams) -> Result<CallToolResult, McpError> {
    let duration_secs = params.duration.unwrap_or(5).clamp(1, 30);
    let device_index = params.index;

    let result = tokio::task::spawn_blocking(move || {
        capture_audio_blocking(device_index, duration_secs)
    })
    .await
    .map_err(|e| internal_error(format!("Task join error: {}", e)))??;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

pub async fn get_input_level(params: LevelParams) -> Result<CallToolResult, McpError> {
    let duration_ms = params.duration_ms.unwrap_or(100).clamp(10, 1000);
    let device_index = params.index;

    let result = tokio::task::spawn_blocking(move || {
        get_input_level_blocking(device_index, duration_ms)
    })
    .await
    .map_err(|e| internal_error(format!("Task join error: {}", e)))??;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

// === Blocking Helper Functions ===

fn get_device_by_index(index: Option<u32>) -> Result<cpal::Device, McpError> {
    let host = cpal::default_host();

    match index {
        None => host.default_input_device().ok_or_else(|| {
            internal_error("No default input device available")
        }),
        Some(idx) => {
            let devices: Vec<_> = host
                .input_devices()
                .map_err(|e| internal_error(format!("Failed to enumerate devices: {}", e)))?
                .collect();

            devices.into_iter().nth(idx as usize).ok_or_else(|| {
                McpError::invalid_params(format!("No microphone at index {}", idx), None)
            })
        }
    }
}

fn capture_audio_blocking(device_index: Option<u32>, duration_secs: u32) -> Result<String, McpError> {
    let device = get_device_by_index(device_index)?;

    let config = device.default_input_config().map_err(|e| {
        internal_error(format!("Failed to get input config: {}", e))
    })?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let sample_format = config.sample_format();

    let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let samples_clone = Arc::clone(&samples);
    let err_flag: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let err_clone = Arc::clone(&err_flag);

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = samples_clone.lock().unwrap();
                buffer.extend_from_slice(data);
            },
            move |err| {
                let mut e = err_clone.lock().unwrap();
                *e = Some(format!("Stream error: {}", err));
            },
            None,
        ),
        cpal::SampleFormat::I16 => {
            let samples_clone = Arc::clone(&samples);
            let err_clone = Arc::clone(&err_flag);
            device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mut buffer = samples_clone.lock().unwrap();
                    buffer.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                },
                move |err| {
                    let mut e = err_clone.lock().unwrap();
                    *e = Some(format!("Stream error: {}", err));
                },
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let samples_clone = Arc::clone(&samples);
            let err_clone = Arc::clone(&err_flag);
            device.build_input_stream(
                &config.into(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let mut buffer = samples_clone.lock().unwrap();
                    buffer.extend(
                        data.iter()
                            .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0),
                    );
                },
                move |err| {
                    let mut e = err_clone.lock().unwrap();
                    *e = Some(format!("Stream error: {}", err));
                },
                None,
            )
        }
        _ => {
            return Err(internal_error(format!("Unsupported sample format: {:?}", sample_format)))
        }
    }
    .map_err(|e| internal_error(format!("Failed to build stream: {}", e)))?;

    stream.play().map_err(|e| {
        internal_error(format!("Failed to start recording: {}", e))
    })?;

    std::thread::sleep(Duration::from_secs(duration_secs as u64));

    drop(stream);

    if let Some(err) = err_flag.lock().unwrap().take() {
        return Err(internal_error(err));
    }

    let recorded_samples = samples.lock().unwrap();

    if recorded_samples.is_empty() {
        return Err(internal_error("No audio data captured"));
    }

    let wav_data = encode_wav(&recorded_samples, sample_rate, channels)?;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&wav_data);

    Ok(format!(
        "Recorded {} seconds of audio ({} samples, {}Hz, {} channels)\n\nBase64 WAV data:\n{}",
        duration_secs,
        recorded_samples.len(),
        sample_rate,
        channels,
        base64_data
    ))
}

fn get_input_level_blocking(device_index: Option<u32>, duration_ms: u32) -> Result<String, McpError> {
    let device = get_device_by_index(device_index)?;

    let config = device.default_input_config().map_err(|e| {
        internal_error(format!("Failed to get input config: {}", e))
    })?;

    let sample_format = config.sample_format();

    let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let samples_clone = Arc::clone(&samples);

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = samples_clone.lock().unwrap();
                buffer.extend_from_slice(data);
            },
            |_| {},
            None,
        ),
        cpal::SampleFormat::I16 => {
            let samples_clone = Arc::clone(&samples);
            device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mut buffer = samples_clone.lock().unwrap();
                    buffer.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                },
                |_| {},
                None,
            )
        }
        _ => {
            return Err(internal_error(format!("Unsupported sample format: {:?}", sample_format)))
        }
    }
    .map_err(|e| internal_error(format!("Failed to build stream: {}", e)))?;

    stream.play().map_err(|e| {
        internal_error(format!("Failed to start stream: {}", e))
    })?;

    std::thread::sleep(Duration::from_millis(duration_ms as u64));

    drop(stream);

    let recorded_samples = samples.lock().unwrap();

    if recorded_samples.is_empty() {
        return Ok(serde_json::to_string_pretty(&InputLevel {
            rms: 0.0,
            peak: 0.0,
            is_silent: true,
        }).unwrap());
    }

    let sum_squares: f32 = recorded_samples.iter().map(|s| s * s).sum();
    let rms = (sum_squares / recorded_samples.len() as f32).sqrt();
    let peak = recorded_samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, |a, b| a.max(b));

    let level = InputLevel {
        rms,
        peak,
        is_silent: rms < 0.01,
    };

    serde_json::to_string_pretty(&level)
        .map_err(|e| internal_error(format!("Serialization error: {}", e)))
}

fn encode_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>, McpError> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| internal_error(format!("Failed to create WAV writer: {}", e)))?;

        for &sample in samples {
            let amplitude = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(amplitude).map_err(|e| {
                internal_error(format!("Failed to write sample: {}", e))
            })?;
        }

        writer.finalize().map_err(|e| {
            internal_error(format!("Failed to finalize WAV: {}", e))
        })?;
    }

    Ok(cursor.into_inner())
}
