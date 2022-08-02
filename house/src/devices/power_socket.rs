use crate::devices::device_info::DeviceInfo;
use crate::DeviceName;

#[derive(Debug, Clone, Copy)]
pub struct PowerSocket {
    pub tpe: SocketType,
    pub voltage: u32,
    pub current: u32,
    pub enabled: bool,
}

impl PowerSocket {
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
                - enabled={}",
            device_name.0,
            &self.tpe,
            &self.voltage.to_string().as_str(),
            &self.current.to_string().as_str(),
            &self.enabled.to_string().as_str(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SocketType {
    A,
    B,
    C,
}
