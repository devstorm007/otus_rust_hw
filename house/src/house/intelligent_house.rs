use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future;
use futures::stream::{self, StreamExt};
use parking_lot::RwLock;

use crate::errors::intelligent_house_error::HouseError;
use crate::errors::intelligent_house_error::HouseError::*;
use crate::house::domain::{DeviceName, HouseName, Room, RoomName};
use crate::house::house_storage::HouseStorage;
use crate::inventory::device_inventory::DeviceInventory;

#[derive(Debug, Clone)]
pub struct IntelligentHouse {
    pub name: HouseName,
    pub rooms: Arc<RwLock<Vec<Room>>>,
}

impl IntelligentHouse {
    pub fn create(name: &str, rooms: Vec<Room>) -> Self {
        IntelligentHouse {
            name: HouseName(name.to_string()),
            rooms: Arc::new(RwLock::new(rooms)),
        }
    }
}

#[async_trait]
impl HouseStorage for IntelligentHouse {
    async fn get_rooms(&self) -> Vec<RoomName> {
        self.rooms
            .read()
            .iter()
            .map(|room| room.name.clone())
            .collect()
    }

    async fn get_room(&self, room_name: &RoomName) -> Option<Room> {
        self.rooms
            .read()
            .iter()
            .find(|r| r.name == *room_name)
            .cloned()
    }

    async fn add_room(&mut self, room_name: &RoomName) -> Result<(), HouseError> {
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

    async fn remove_room(&mut self, room_name: &RoomName) -> Result<(), HouseError> {
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

    async fn remove_device(
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

    async fn generate_report<T: DeviceInventory + Sync>(
        &self,
        inventory: &T,
    ) -> Result<String, HouseError> {
        let room_names = self.get_rooms().await;

        let prefix_msg = format!("'{}' contains {} rooms:\n", self.name.0, room_names.len());

        /*let result =
        future::try_join_all(
            ids.iter().map(|id| Foo::get(*id))
        )
            .await
            .unwrap();*/

        /*let a = future::ok::<i32, i32>(1);
        assert_eq!(a.await, Ok(1));*/

        //let sum = number_stream.fold(0, |acc, x| async move { acc + x });

        //let it = stream::iter(room_names);

        /*let res = room_names.iter().fold(prefix_msg, |house_info, room_name| {
            let device_names = self.get_devices(room_name).await?;

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
        });*/

        Ok("".to_string())
    }
}
