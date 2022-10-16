use house::devices::power_socket::{PowerSocket, SocketType};
use house::house::domain::*;
use house::house::house_storage::HouseStorage;
use house::inventory::memory_device_inventory::DeviceItem;
use house::synchronizer::device_synchronizer::{DeviceSynchronizer, HouseDeviceSynchronizer};
use house::{mk_three_rooms_house, mk_three_rooms_inventory, ThreeRoomNames};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let room_device_names = ThreeRoomNames::default();

    let house = Arc::new(Mutex::new(mk_three_rooms_house(room_device_names.clone())));
    let inventory = mk_three_rooms_inventory(room_device_names.clone());

    let _report = house
        .lock()
        .await
        .generate_report(&inventory)
        .await
        .unwrap();
    println!("{_report}");

    let mut sync = HouseDeviceSynchronizer::new(house.clone(), inventory.clone());

    let children_room = &RoomName("children".to_string());
    sync.add_room(children_room).await.unwrap();

    let device_name = DeviceName("extra ps 220".to_string());
    let device = DeviceItem::inject(PowerSocket {
        tpe: SocketType::C,
        voltage: 220,
        current: 10,
        enabled: true,
    });

    sync.add_device(children_room, &device_name, device)
        .await
        .unwrap();

    sync.remove_device(&room_device_names.lounge, &room_device_names.socket2)
        .await
        .unwrap();

    sync.remove_room(&room_device_names.bedroom).await.unwrap();

    let _report_after_change = house
        .lock()
        .await
        .generate_report(&inventory)
        .await
        .unwrap();
    println!("{_report_after_change}");
}
