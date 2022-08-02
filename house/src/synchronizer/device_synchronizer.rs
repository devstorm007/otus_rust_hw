use anyhow::Result;

use crate::errors::app_error::AppError;
use crate::house::intelligent_house::*;
use crate::inventory::device_inventory::DeviceInventory;
use crate::DeviceItem;

pub trait DeviceSynchronizer {
  fn add_room(&mut self, room_name: &RoomName) -> Result<(), AppError>;

  fn remove_room(&mut self, room_name: &RoomName) -> Result<(), AppError>;

  fn add_device(
    &mut self,
    room_name: &RoomName,
    device_name: &DeviceName,
    device: DeviceItem,
  ) -> Result<(), AppError>;

  fn remove_device(
    &mut self,
    room_name: &RoomName,
    device_name: &DeviceName,
  ) -> Result<(), AppError>;
}

pub struct HouseDeviceSynchronizer<'a, T: DeviceInventory> {
  house: &'a mut IntelligentHouse,
  inventory: &'a mut T,
}

impl<'a, T: DeviceInventory> HouseDeviceSynchronizer<'a, T> {
  pub fn new(
    house: &'a mut IntelligentHouse,
    inventory: &'a mut T,
  ) -> HouseDeviceSynchronizer<'a, T> {
    HouseDeviceSynchronizer { house, inventory }
  }
}

impl<'a, T: DeviceInventory> DeviceSynchronizer for HouseDeviceSynchronizer<'a, T> {
  fn add_room(&mut self, room_name: &RoomName) -> Result<(), AppError> {
    self.house.add_room(room_name)?;
    self.inventory.add_room(room_name)?;
    Ok(())
  }

  fn remove_room(&mut self, room_name: &RoomName) -> Result<(), AppError> {
    self.house.remove_room(room_name)?;
    self.inventory.remove_room(room_name)?;
    Ok(())
  }

  fn add_device(
    &mut self,
    room_name: &RoomName,
    device_name: &DeviceName,
    device: DeviceItem,
  ) -> Result<(), AppError> {
    self.house.add_device(room_name, device_name)?;
    self.inventory.add_device(room_name, device_name, device)?;
    Ok(())
  }

  fn remove_device(
    &mut self,
    room_name: &RoomName,
    device_name: &DeviceName,
  ) -> Result<(), AppError> {
    self.house.remove_device(room_name, device_name)?;
    self.inventory.remove_device(room_name, device_name)?;
    Ok(())
  }
}
