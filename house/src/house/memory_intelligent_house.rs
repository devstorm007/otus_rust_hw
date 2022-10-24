use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use parking_lot::RwLock;

use crate::errors::intelligent_house_error::HouseError;
use crate::errors::intelligent_house_error::HouseError::*;
use crate::house::domain::{DeviceName, HouseName, Room, RoomName};
use crate::house::intelligent_house::IntelligentHouse;
use crate::inventory::device_inventory::DeviceInventory;

#[derive(Debug, Clone)]
pub struct MemoryIntelligentHouse {
    pub name: HouseName,
    pub rooms: Arc<RwLock<Vec<Room>>>,
}

impl MemoryIntelligentHouse {
    pub fn create(name: &str, rooms: Vec<Room>) -> Self {
        MemoryIntelligentHouse {
            name: HouseName(name.to_string()),
            rooms: Arc::new(RwLock::new(rooms)),
        }
    }
}

#[async_trait]
impl IntelligentHouse for MemoryIntelligentHouse {
    async fn get_rooms(&self) -> Result<Vec<Room>, HouseError> {
        Ok(self.rooms.read().iter().cloned().collect())
    }

    async fn get_room(&self, room_name: &RoomName) -> Result<Room, HouseError> {
        let room_opt = self
            .rooms
            .read()
            .iter()
            .find(|r| r.name == *room_name)
            .cloned();

        room_opt.ok_or_else(|| RoomNotFound(room_name.clone()))
    }

    async fn add_room(&self, room_name: &RoomName) -> Result<(), HouseError> {
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

    async fn remove_room(&self, room_name: &RoomName) -> Result<(), HouseError> {
        let mut rooms = self.rooms.write();
        rooms
            .iter()
            .position(|r| r.name == *room_name)
            .map(|index| {
                rooms.swap_remove(index);
            })
            .ok_or_else(|| RoomNotFound(room_name.clone()))
    }

    async fn get_devices(&self, room_name: &RoomName) -> Result<Vec<DeviceName>, HouseError> {
        self.rooms
            .read()
            .iter()
            .find(|r| r.name == *room_name)
            .map(|r| r.devices.clone())
            .ok_or_else(|| RoomNotFound(room_name.clone()))
    }

    async fn add_device(
        &self,
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

    async fn remove_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), HouseError> {
        let mut rooms = self.rooms.write();
        let room = rooms
            .iter_mut()
            .find(|r| r.name == *room_name)
            .ok_or_else(|| RoomNotFound(room_name.clone()))?;

        room.devices
            .iter()
            .position(|d| d == device_name)
            .map(|index| {
                room.devices.swap_remove(index);
            })
            .ok_or_else(|| RoomDeviceNotFound(device_name.clone(), room_name.clone()))
    }

    async fn generate_report<T: DeviceInventory + Sync>(
        &self,
        inventory: &T,
    ) -> Result<String, HouseError> {
        let room_names: Vec<RoomName> = self
            .get_rooms()
            .await?
            .iter()
            .map(|room| room.name.clone())
            .collect();

        let prefix_msg = format!("'{}' contains {} rooms:\n", self.name.0, room_names.len());

        stream::iter(room_names)
            .fold(Ok(prefix_msg), |house_info, room_name| async move {
                let device_names = self.get_devices(&room_name).await?;

                let (devices_info, _) = stream::iter(device_names)
                    .fold(
                        ("".to_string(), room_name.clone()),
                        |(acc_dev_info, rn), device_name| async move {
                            let _dev_info: String = inventory
                                .get_info(&rn, &device_name)
                                .await
                                .unwrap_or_else(|e| format!("{e}"));

                            (format!("{acc_dev_info}     {_dev_info}\n"), rn)
                        },
                    )
                    .await;

                Ok(format!(
                    "{}   '{}' has:\n{}\n",
                    house_info?, room_name.0, devices_info
                ))
            })
            .await
    }
}
