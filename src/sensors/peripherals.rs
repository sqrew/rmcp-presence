//! Peripherals - all connected hardware in one call

use rmcp::{model::{CallToolResult, Content}, ErrorData as McpError};
use serde::Serialize;

use crate::shared::internal_error;

#[derive(Debug, Serialize)]
pub struct Peripherals {
    pub displays: Vec<DisplayInfo>,
    pub usb_devices: Vec<UsbDevice>,
    pub cameras: Vec<CameraInfo>,
    pub microphones: Vec<MicrophoneInfo>,
    #[cfg(all(feature = "linux", target_os = "linux"))]
    pub bluetooth: BluetoothStatus,
}

#[derive(Debug, Serialize)]
pub struct DisplayInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[derive(Debug, Serialize)]
pub struct UsbDevice {
    pub name: String,
    pub vendor_id: String,
    pub product_id: String,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CameraInfo {
    pub index: u32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct MicrophoneInfo {
    pub index: u32,
    pub name: String,
    pub is_default: bool,
}

#[cfg(all(feature = "linux", target_os = "linux"))]
#[derive(Debug, Serialize)]
pub struct BluetoothStatus {
    pub adapter: Option<String>,
    pub powered: bool,
    pub connected_devices: Vec<BluetoothDevice>,
}

#[cfg(all(feature = "linux", target_os = "linux"))]
#[derive(Debug, Serialize)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    pub connected: bool,
}

pub async fn get_peripherals() -> Result<CallToolResult, McpError> {
    let peripherals = build_peripherals().await.map_err(|e| internal_error(e.to_string()))?;
    let json = serde_json::to_string_pretty(&peripherals).map_err(|e| internal_error(e.to_string()))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

async fn build_peripherals() -> anyhow::Result<Peripherals> {
    // Displays
    let displays = get_displays();

    // USB
    let usb_devices = get_usb_devices();

    // Cameras
    let cameras = get_cameras();

    // Microphones
    let microphones = get_microphones();

    // Bluetooth (Linux only)
    #[cfg(all(feature = "linux", target_os = "linux"))]
    let bluetooth = get_bluetooth().await;

    Ok(Peripherals {
        displays,
        usb_devices,
        cameras,
        microphones,
        #[cfg(all(feature = "linux", target_os = "linux"))]
        bluetooth,
    })
}

fn get_displays() -> Vec<DisplayInfo> {
    display_info::DisplayInfo::all()
        .unwrap_or_default()
        .into_iter()
        .map(|d| DisplayInfo {
            name: d.name,
            width: d.width,
            height: d.height,
            is_primary: d.is_primary,
        })
        .collect()
}

fn get_usb_devices() -> Vec<UsbDevice> {
    match nusb::list_devices() {
        Ok(devices) => devices
            .map(|d| UsbDevice {
                name: d.product_string().unwrap_or_default().to_string(),
                vendor_id: format!("{:04x}", d.vendor_id()),
                product_id: format!("{:04x}", d.product_id()),
                manufacturer: d.manufacturer_string().map(|s| s.to_string()),
                product: d.product_string().map(|s| s.to_string()),
            })
            .collect(),
        Err(_) => vec![],
    }
}

fn get_cameras() -> Vec<CameraInfo> {
    use nokhwa::utils::CameraIndex;

    nokhwa::query(nokhwa::utils::ApiBackend::Auto)
        .unwrap_or_default()
        .into_iter()
        .map(|c| {
            let index = match c.index() {
                CameraIndex::Index(i) => *i,
                CameraIndex::String(_) => 0,
            };
            CameraInfo {
                index,
                name: c.human_name().to_string(),
            }
        })
        .collect()
}

fn get_microphones() -> Vec<MicrophoneInfo> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    host.input_devices()
        .map(|devices| {
            devices
                .enumerate()
                .filter_map(|(idx, device)| {
                    let name = device.name().ok()?;
                    let is_default = default_name.as_ref() == Some(&name);
                    Some(MicrophoneInfo {
                        index: idx as u32,
                        name,
                        is_default,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(all(feature = "linux", target_os = "linux"))]
async fn get_bluetooth() -> BluetoothStatus {
    use bluer::{Adapter, Session};

    let session = match Session::new().await {
        Ok(s) => s,
        Err(_) => return BluetoothStatus {
            adapter: None,
            powered: false,
            connected_devices: vec![],
        },
    };

    let adapter_name = session.default_adapter().await.ok().map(|a| a.name().to_string());

    let adapter = match session.default_adapter().await {
        Ok(a) => a,
        Err(_) => return BluetoothStatus {
            adapter: adapter_name,
            powered: false,
            connected_devices: vec![],
        },
    };

    let powered = adapter.is_powered().await.unwrap_or(false);

    let mut connected_devices = vec![];
    if let Ok(addrs) = adapter.device_addresses().await {
        for addr in addrs {
            if let Ok(device) = adapter.device(addr) {
                let connected = device.is_connected().await.unwrap_or(false);
                let name = device.name().await.ok().flatten().unwrap_or_else(|| addr.to_string());
                connected_devices.push(BluetoothDevice {
                    name,
                    address: addr.to_string(),
                    connected,
                });
            }
        }
    }

    BluetoothStatus {
        adapter: Some(adapter.name().to_string()),
        powered,
        connected_devices,
    }
}
