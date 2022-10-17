use std::fmt::Debug;

use async_trait::async_trait;
use mongodb::Database;

use house::errors::intelligent_house_error::HouseError;
use house::house::domain::{DeviceName, Room, RoomName};
use house::house::intelligent_house::IntelligentHouse;
use house::inventory::device_inventory::DeviceInventory;

#[derive(Debug, Clone)]
pub struct DbIntelligentHouse {
    db: Database,
}

impl DbIntelligentHouse {
    pub fn new(db: Database) -> Self {
        DbIntelligentHouse { db }
    }
}

#[async_trait]
impl IntelligentHouse for DbIntelligentHouse {
    async fn get_rooms(&self) -> Vec<RoomName> {
        //self.db.collection::<NewUser>("rooms")
        /*mongo
        .collection::<NewUser>("users")
        .insert_one(self, None)
        .await
        .map_err(|err| format!("DB_ERROR: {}", err))?;*/
    }

    async fn get_room(&self, room_name: &RoomName) -> Option<Room> {
        todo!()
    }

    async fn add_room(&mut self, room_name: &RoomName) -> Result<(), HouseError> {
        todo!()
    }

    async fn remove_room(&mut self, room_name: &RoomName) -> Result<(), HouseError> {
        todo!()
    }

    async fn get_devices(&self, room_name: &RoomName) -> Result<Vec<DeviceName>, HouseError> {
        todo!()
    }

    async fn add_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), HouseError> {
        todo!()
    }

    async fn remove_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<DeviceName, HouseError> {
        todo!()
    }

    async fn generate_report<T: DeviceInventory + Sync>(
        &self,
        inventory: &T,
    ) -> Result<String, HouseError> {
        todo!()
    }
}
