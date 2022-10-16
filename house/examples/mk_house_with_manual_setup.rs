use std::collections::HashMap;

use house::devices::power_socket::{PowerSocket, SocketType};
use house::house::domain::*;
use house::house::house_storage::HouseStorage;
use house::house::intelligent_house::IntelligentHouse;
use house::inventory::memory_device_inventory::{DeviceItem, MemoryDeviceInventory};

#[tokio::main]
async fn main() {
    let kitchen_name = RoomName("kitchen".to_string());
    let kitchen = Room {
        name: kitchen_name.clone(),
        devices: Vec::from([DeviceName("socket 220V".to_string())]),
    };
    let house: IntelligentHouse = IntelligentHouse::create("kitchen house", Vec::from([kitchen]));

    let power_sockets = HashMap::from([(
        kitchen_name,
        HashMap::from([(
            DeviceName("socket 220V".to_string()),
            DeviceItem::inject(PowerSocket {
                tpe: SocketType::C,
                voltage: 220,
                current: 5,
                enabled: true,
            }),
        )]),
    )]);

    let inventory: MemoryDeviceInventory = MemoryDeviceInventory::new(power_sockets);

    let _report = house.generate_report(&inventory).await.unwrap();

    println!("{_report}");
}
