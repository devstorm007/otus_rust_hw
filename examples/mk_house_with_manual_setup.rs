use sh_mod::house::intelligent_house::IntelligentHouse;
use sh_mod::*;
use sh_mod::inventory::memory_device_inventory::MemoryDeviceInventory;

fn main() {
    let mut empty_house: IntelligentHouse = Default::default();
    let mut empty_inventory: MemoryDeviceInventory = Default::default();

    /*let inv1r = mk_one_room_inventory();
    let _h1r_report = h1r.generate_report(&inv1r);

    let h3r_names = ThreeRoomNames::default();
    let h3r = mk_three_rooms_house(h3r_names.clone());
    let inv3r = mk_three_rooms_inventory(h3r_names);
    let _h3r_report = h3r.generate_report(&inv3r);

    println!("{_h1r_report}\n{_h3r_report}");*/
}

/*impl Default for MemoryDeviceInventory {
    fn default() -> Self {
        let power_sockets = HashMap::from([(
            RoomName("kitchen".to_string()),
            HashMap::from([(
                DeviceName("socket 220V".to_string()),
                PowerSocket {
                    tpe: SocketType::C,
                    voltage: 220,
                    current: 5,
                    enabled: true,
                },
            )]),
        )]);

        MemoryDeviceInventory {
            room_power_sockets: power_sockets,
            room_temperature_sensors: Default::default(),
        }
    }
}*/
/*impl Default for IntelligentHouse {
    fn default() -> Self {
        let kitchen = Room {
            name: RoomName("kitchen".to_string()),
            devices: Vec::from([DeviceName("socket 220V".to_string())]),
        };
        IntelligentHouse {
            name: HouseName("kitchen house".to_string()),
            rooms: Vec::from([kitchen]),
        }
    }
}*/