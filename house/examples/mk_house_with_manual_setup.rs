use std::collections::HashMap;

use house::devices::power_socket::{PowerSocket, SocketType};
use house::house::domain::*;
use house::house::intelligent_house::IntelligentHouse;
use house::house::memory_intelligent_house::MemoryIntelligentHouse;
use house::inventory::domain::DeviceItem;
use house::inventory::memory_device_inventory::MemoryDeviceInventory;

#[tokio::main]
async fn main() {
    let kitchen_name = RoomName("kitchen".to_string());
    let kitchen = Room {
        name: kitchen_name.clone(),
        devices: Vec::from([DeviceName("socket 220V".to_string())]),
    };
    let house: MemoryIntelligentHouse =
        MemoryIntelligentHouse::create("kitchen house", Vec::from([kitchen]));

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
