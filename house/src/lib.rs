extern crate derive_more;
extern crate frunk;

use std::collections::HashMap;

use crate::devices::power_socket::{PowerSocket, SocketType};
use crate::devices::temperature_sensor::{SensorRange, TemperatureSensor};
use crate::house::intelligent_house::*;
use crate::inventory::memory_device_inventory::{DeviceItem, MemoryDeviceInventory};

pub mod devices;
pub mod errors;
pub mod house;
pub mod inventory;
pub mod synchronizer;

#[derive(Clone)]
pub struct ThreeRoomNames {
    pub lounge: RoomName,
    pub bedroom: RoomName,
    pub kitchen: RoomName,
    pub socket1: DeviceName,
    pub socket2: DeviceName,
    pub socket3: DeviceName,
    pub socket4: DeviceName,
    pub sensor1: DeviceName,
}

impl Default for ThreeRoomNames {
    fn default() -> Self {
        ThreeRoomNames {
            lounge: RoomName("lounge".to_string()),
            bedroom: RoomName("bedroom".to_string()),
            kitchen: RoomName("kitchen".to_string()),
            socket1: DeviceName("socket1".to_string()),
            socket2: DeviceName("socket2".to_string()),
            socket3: DeviceName("socket3".to_string()),
            socket4: DeviceName("socket4".to_string()),
            sensor1: DeviceName("sensor1".to_string()),
        }
    }
}

pub fn mk_three_rooms_inventory(names: ThreeRoomNames) -> MemoryDeviceInventory {
    let kitchen_name = names.kitchen;
    let devices = HashMap::from([
        (
            names.bedroom,
            HashMap::from([(
                names.socket1,
                DeviceItem::inject(PowerSocket {
                    tpe: SocketType::C,
                    voltage: 220,
                    current: 15,
                    enabled: true,
                }),
            )]),
        ),
        (
            names.lounge,
            HashMap::from([
                (
                    names.socket2,
                    DeviceItem::inject(PowerSocket {
                        tpe: SocketType::B,
                        voltage: 220,
                        current: 15,
                        enabled: true,
                    }),
                ),
                (
                    names.socket3,
                    DeviceItem::inject(PowerSocket {
                        tpe: SocketType::A,
                        voltage: 230,
                        current: 10,
                        enabled: true,
                    }),
                ),
            ]),
        ),
        (
            kitchen_name,
            HashMap::from([
                (
                    names.socket4,
                    DeviceItem::inject(PowerSocket {
                        tpe: SocketType::B,
                        voltage: 250,
                        current: 20,
                        enabled: true,
                    }),
                ),
                (
                    names.sensor1,
                    DeviceItem::inject(TemperatureSensor {
                        temperature: 26,
                        range: SensorRange { min: 10, max: 40 },
                        accuracy: 1,
                    }),
                ),
            ]),
        ),
    ]);

    MemoryDeviceInventory::new(devices)
}

pub fn mk_three_rooms_house(names: ThreeRoomNames) -> IntelligentHouse {
    let bedroom = Room {
        name: names.bedroom,
        devices: vec![names.socket1],
    };
    let lounge = Room {
        name: names.lounge,
        devices: vec![names.socket2, names.socket3],
    };
    let kitchen = Room {
        name: names.kitchen,
        devices: vec![names.socket4, names.sensor1],
    };

    IntelligentHouse::from("bachelor's house", vec![bedroom, kitchen, lounge])
}
