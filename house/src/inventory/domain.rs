use crate::{DeviceName, RoomName};
use frunk::Coprod;
use frunk_core::hlist;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::devices::device_info::DeviceInfo;
use crate::devices::power_socket::PowerSocket;
use crate::devices::temperature_sensor::TemperatureSensor;

pub type DeviceItem = Coprod!(PowerSocket, TemperatureSensor);

pub fn get_info(device: DeviceItem, device_name: &DeviceName) -> String {
    device.fold(hlist![
        |ps: PowerSocket| ps.get_info(device_name),
        |ts: TemperatureSensor| ts.get_info(device_name)
    ])
}

#[derive(Eq, PartialEq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoomDevices {
    pub name: RoomName,
    pub sockets: HashMap<DeviceName, PowerSocket>,
    pub sensors: HashMap<DeviceName, TemperatureSensor>,
}
