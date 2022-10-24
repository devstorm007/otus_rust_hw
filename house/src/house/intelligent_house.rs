use async_trait::async_trait;

use crate::errors::intelligent_house_error::HouseError;
use crate::house::domain::{DeviceName, Room, RoomName};
use crate::inventory::device_inventory::DeviceInventory;

#[async_trait]
pub trait IntelligentHouse {
    async fn get_rooms(&self) -> Result<Vec<Room>, HouseError>;

    async fn get_room(&self, room_name: &RoomName) -> Result<Room, HouseError>;

    async fn add_room(&self, room_name: &RoomName) -> Result<(), HouseError>;

    async fn remove_room(&self, room_name: &RoomName) -> Result<(), HouseError>;

    async fn get_devices(&self, room_name: &RoomName) -> Result<Vec<DeviceName>, HouseError>;

    async fn add_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), HouseError>;

    async fn remove_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), HouseError>;

    async fn generate_report<T: DeviceInventory + Sync>(
        &self,
        inventory: &T,
    ) -> Result<String, HouseError>;
}
