use crate::inv::InvFactory;
use house::house::domain::{DeviceName, RoomName};
use std::error::Error;

mod inv;

fn main() -> Result<(), Box<dyn Error>> {
    let inv_factory = InvFactory::new()?;

    let mut device_inventory = inv_factory.create_inventory()?;

    let room_name = &RoomName("kitchen".to_string());
    let device_name = &DeviceName("socket 220V-5A".to_string());

    println!("get_socket_info");
    let initial_info = device_inventory.get_socket_info(room_name, device_name)?;
    println!("initial info: {initial_info}");

    device_inventory.disable_socket(room_name, device_name)?;
    let disabled_info = device_inventory.get_socket_info(room_name, device_name)?;
    println!("disabled info: {disabled_info}");

    device_inventory.enable_socket(room_name, device_name)?;
    let enabled_info = device_inventory.get_socket_info(room_name, device_name)?;

    println!("enabled info: {enabled_info}");

    Ok(())
}
