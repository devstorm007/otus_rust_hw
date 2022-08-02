use anyhow::Result;

use crate::errors::app_error::InventoryError;
use crate::house::intelligent_house::*;
use crate::DeviceItem;

pub trait DeviceInventory {
  fn get_info(
    &self,
    room_name: &RoomName,
    device_name: &DeviceName,
  ) -> Result<String, InventoryError>;

  fn add_room(&self, room_name: &RoomName) -> Result<(), InventoryError>;

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
}
