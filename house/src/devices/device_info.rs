use crate::DeviceName;

pub trait DeviceInfo {
    fn get_info(&self, device_name: &DeviceName) -> String;
}
