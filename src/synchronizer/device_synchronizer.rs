use crate::errors::app_error::InventoryError;
use crate::house::intelligent_house::*;
use anyhow::Result;

pub trait DeviceInventory {
    fn get_info(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<String, InventoryError>;

    fn remove_room(&self, room_name: &RoomName) -> Result<(), InventoryError>;

    fn remove_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), InventoryError>;
}
