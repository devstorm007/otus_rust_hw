use crate::devices::device_info::DeviceInfo;
use crate::DeviceName;
use rand::prelude::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemperatureSensor {
    pub temperature: i32,
    pub range: SensorRange,
    pub accuracy: i32,
}

impl TemperatureSensor {
    pub fn current_temperature(&self) -> i32 {
        let mut rng: ThreadRng = rand::thread_rng();
        rng.gen_range(self.range.min..self.range.max)
    }
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

#[derive(Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SensorRange {
    pub min: i32,
    pub max: i32,
}
