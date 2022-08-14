use std::fmt::Debug;
use std::hash::Hash;

use derive_more::Display;
use parking_lot::RwLock;

use crate::errors::intelligent_house_error::HouseError;
use crate::errors::intelligent_house_error::HouseError::*;
use crate::inventory::device_inventory::DeviceInventory;

#[derive(Debug)]
pub struct IntelligentHouse {
    pub name: HouseName,
    pub rooms: RwLock<Vec<Room>>,
}

impl IntelligentHouse {
    pub fn from(name: &str, rooms: Vec<Room>) -> Self {
        IntelligentHouse {
            name: HouseName(name.to_string()),
            rooms: RwLock::new(rooms),
        }
    }

    pub fn get_rooms(&self) -> Vec<RoomName> {
        self.rooms
            .read()
            .iter()
            .map(|room| room.name.clone())
            .collect()
    }

    pub fn get_room(&self, room_name: &RoomName) -> Option<Room> {
        self.rooms
            .read()
            .iter()
            .find(|r| r.name == *room_name)
            .cloned()
    }

    pub fn add_room(&mut self, room_name: &RoomName) -> Result<(), HouseError> {
        let mut rooms = self.rooms.write();
        if rooms.iter().any(|r| r.name == *room_name) {
            Err(RoomAlreadyAdded(room_name.clone()))
        } else {
            rooms.push(Room {
                name: room_name.clone(),
                devices: Vec::new(),
            });
            Ok(())
        }
    }

    pub fn remove_room(&mut self, room_name: &RoomName) -> Result<(), HouseError> {
        let mut rooms = self.rooms.write();
        rooms
            .iter()
            .position(|r| r.name == *room_name)
            .map(|index| {
                rooms.swap_remove(index);
            })
            .ok_or_else(|| RoomNotFound(room_name.clone()))
    }

    pub fn get_devices(&self, room_name: &RoomName) -> Result<Vec<DeviceName>, HouseError> {
        self.rooms
            .read()
            .iter()
            .find(|r| r.name == *room_name)
            .map(|r| r.devices.clone())
            .ok_or_else(|| RoomNotFound(room_name.clone()))
    }

    pub fn add_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), HouseError> {
        let mut rooms = self.rooms.write();
        let room = rooms
            .iter_mut()
            .find(|r| r.name == *room_name)
            .ok_or_else(|| RoomNotFound(room_name.clone()))?;

        if room.devices.iter().any(|d| d == device_name) {
            Err(RoomDeviceAlreadyAdded(
                device_name.clone(),
                room_name.clone(),
            ))
        } else {
            room.devices.push(device_name.clone());
            Ok(())
        }
    }

    pub fn remove_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<DeviceName, HouseError> {
        let mut rooms = self.rooms.write();
        let room = rooms
            .iter_mut()
            .find(|r| r.name == *room_name)
            .ok_or_else(|| RoomNotFound(room_name.clone()))?;

        room.devices
            .iter()
            .position(|d| d == device_name)
            .map(|index| room.devices.swap_remove(index))
            .ok_or_else(|| RoomDeviceNotFound(device_name.clone(), room_name.clone()))
    }

    pub fn generate_report<T: DeviceInventory>(&self, inventory: &T) -> Result<String, HouseError> {
        let room_names = self.get_rooms();

        let prefix_msg = format!("'{}' contains {} rooms:\n", self.name.0, room_names.len());

        room_names
            .iter()
            .fold(Ok(prefix_msg), |house_info, room_name| {
                let device_names = self.get_devices(room_name)?;

                let devices_info =
                    device_names
                        .iter()
                        .fold("".to_string(), |acc_dev_info, device_name| {
                            let _dev_info: String = inventory
                                .get_info(room_name, device_name)
                                .unwrap_or_else(|e| format!("{e}"));

                            format!("{acc_dev_info}     {_dev_info}\n")
                        });

                Ok(format!(
                    "{}   '{}' has:\n{}\n",
                    house_info?, room_name.0, devices_info
                ))
            })
    }
}

#[derive(Debug, Clone)]
pub struct Room {
    pub name: RoomName,
    pub devices: Vec<DeviceName>,
}

#[derive(Debug, Display)]
pub struct HouseName(pub String);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Display)]
pub struct RoomName(pub String);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Display)]
pub struct DeviceName(pub String);
