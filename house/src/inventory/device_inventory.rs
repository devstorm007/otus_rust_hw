use async_trait::async_trait;

use crate::errors::intelligent_house_error::InventoryError;
use crate::house::domain::*;
use crate::inventory::domain::{DeviceItem, RoomDevices};

#[async_trait]
pub trait DeviceInventory {
    async fn get_info(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<String, InventoryError>;

    async fn get_rooms(&self) -> Result<Vec<RoomName>, InventoryError>;

    async fn add_room(&self, room_name: &RoomName) -> Result<(), InventoryError>;

    async fn remove_room(&self, room_name: &RoomName) -> Result<(), InventoryError>;

    async fn get_all_room_devices(&self) -> Result<Vec<RoomDevices>, InventoryError>;

    async fn add_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
        device: DeviceItem,
    ) -> Result<(), InventoryError>;

    async fn remove_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), InventoryError>;

    async fn change_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
        modify: impl Fn(DeviceItem) -> Result<DeviceItem, InventoryError> + Send,
    ) -> Result<(), InventoryError>;

    async fn get_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<DeviceItem, InventoryError>;
}
