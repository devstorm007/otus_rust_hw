use async_trait::async_trait;

use crate::errors::intelligent_house_error::InventoryError;
use crate::house::intelligent_house::*;
use crate::DeviceItem;

#[async_trait]
pub trait DeviceInventory {
    fn get_info(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<String, InventoryError>;

    async fn get_rooms(&self) -> Result<Vec<RoomName>, InventoryError>;

    async fn add_room(&self, room_name: &RoomName) -> Result<(), InventoryError>;

    fn remove_room(&self, room_name: &RoomName) -> Result<(), InventoryError>;

    fn add_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
        device: DeviceItem,
    ) -> Result<(), InventoryError>;

    fn remove_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), InventoryError>;

    fn change_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
        modify: impl Fn(DeviceItem) -> Result<DeviceItem, InventoryError>,
    ) -> Result<(), InventoryError>;

    fn get_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<DeviceItem, InventoryError>;
}
