use crate::devices::device_info::DeviceInfo;
use crate::DeviceName;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PowerSocket {
    pub tpe: SocketType,
    pub voltage: u32,
    pub current: u32,
    pub enabled: bool,
}

impl PowerSocket {
    pub fn power(self) -> u32 {
        self.voltage * self.current
    }
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl Default for PowerSocket {
    fn default() -> Self {
        PowerSocket {
            tpe: SocketType::C,
            voltage: 220,
            current: 10,
            enabled: true,
        }
    }
}

impl DeviceInfo for PowerSocket {
    fn get_info(&self, device_name: &DeviceName) -> String {
        format!(
            "PS '{}' specification:
                - type {:?}
                - voltage={}
                - current={}
                - enabled={}
                - power={}",
            device_name.0,
            &self.tpe,
            &self.voltage.to_string().as_str(),
            &self.current.to_string().as_str(),
            &self.enabled.to_string().as_str(),
            &self.power().to_string().as_str(),
        )
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SocketType {
    A,
    B,
    C,
}
