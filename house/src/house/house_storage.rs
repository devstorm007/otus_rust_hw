use crate::errors::intelligent_house_error::HouseError;
use crate::house::domain::{DeviceName, Room, RoomName};
use crate::inventory::device_inventory::DeviceInventory;

pub trait HouseStorage {
    fn get_rooms(&self) -> Vec<RoomName>;
    fn get_room(&self, room_name: &RoomName) -> Option<Room>;
    fn add_room(&mut self, room_name: &RoomName) -> Result<(), HouseError>;
    fn remove_room(&mut self, room_name: &RoomName) -> Result<(), HouseError>;
    fn get_devices(&self, room_name: &RoomName) -> Result<Vec<DeviceName>, HouseError>;
    fn add_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), HouseError>;
    fn remove_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<DeviceName, HouseError>;
    fn generate_report<T: DeviceInventory>(&self, inventory: &T) -> Result<String, HouseError>;
}
