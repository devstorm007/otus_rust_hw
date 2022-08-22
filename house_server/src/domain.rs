use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMessage {
    pub body: RequestBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestBody {
    ChangeDeviceData {
        location: DeviceLocation,
        data: DeviceData,
    },
    ShowDeviceInfo {
        location: DeviceLocation,
    },
    RegisterDeviceMonitor {
        location: DeviceLocation,
    },
    RemoveDeviceMonitor,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceLocation {
    pub room_name: String,
    pub device_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceData {
    PowerSocketState { enabled: bool },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub body: ResponseBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseBody {
    DeviceDataChanged,
    DeviceDescription(String),
    MonitorRegistered,
    MonitorRemoved,
    PowerSocketInfo { enabled: bool, power: u32 },
    TemperatureSensorInfo { temperature: i32 },
}
