use crate::devices::device_info::DeviceInfo;
use crate::DeviceName;

#[derive(Debug, Clone, Copy)]
pub struct TemperatureSensor {
  pub temperature: i32,
  pub range: SensorRange,
  pub accuracy: i32,
}

impl DeviceInfo for TemperatureSensor {
  fn get_info(&self, device_name: &DeviceName) -> String {
    format!(
      "TS '{}' specification:
                - {}C° 
                - range {}-{}C° 
                - accuracy {}",
      device_name.0,
      &self.temperature.to_string().as_str(),
      &self.range.min.to_string().as_str(),
      &self.range.max.to_string().as_str(),
      &self.accuracy.to_string().as_str()
    )
  }
}

#[derive(Debug, Clone, Copy)]
pub struct SensorRange {
  pub min: i32,
  pub max: i32,
}
