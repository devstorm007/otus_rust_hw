use house::devices::power_socket::{PowerSocket, SocketType};
use house::house::intelligent_house::{DeviceName, RoomName};
use house::inventory::memory_device_inventory::DeviceItem;
use house::synchronizer::device_synchronizer::{DeviceSynchronizer, HouseDeviceSynchronizer};
use house::{mk_three_rooms_house, mk_three_rooms_inventory, ThreeRoomNames};

fn main() {
    let room_device_names = ThreeRoomNames::default();

    let mut house = mk_three_rooms_house(room_device_names.clone());
    let mut inventory = mk_three_rooms_inventory(room_device_names.clone());

    let _report = &house.generate_report(&inventory).unwrap();
    println!("{_report}");

    let mut sync = HouseDeviceSynchronizer::new(&mut house, &mut inventory);
    let children_room = &RoomName("children".to_string());
    sync.add_room(children_room).unwrap();

    let device_name = DeviceName("extra ps 220".to_string());
    let device = DeviceItem::inject(PowerSocket {
        tpe: SocketType::C,
        voltage: 220,
        current: 10,
        enabled: true,
    });

    sync.add_device(children_room, &device_name, device)
        .unwrap();

    sync.remove_device(&room_device_names.lounge, &room_device_names.socket2)
        .unwrap();

    sync.remove_room(&room_device_names.bedroom).unwrap();

    let _report_after_change = &house.generate_report(&inventory).unwrap();
    println!("{_report_after_change}");
}
//examples/mk_predefined_house.rs
