use std::collections::HashMap;

use house::devices::device_info::DeviceInfo;
use house::devices::power_socket::*;
use house::devices::temperature_sensor::{SensorRange, TemperatureSensor};
use house::house::intelligent_house::*;
use house::inventory::device_inventory::DeviceInventory;
use house::inventory::memory_device_inventory::{DeviceItem, MemoryDeviceInventory};

#[test]
fn test_socket_info() {
    let device_name = DeviceName("socket1".to_string());

    let ps = PowerSocket {
        tpe: SocketType::C,
        voltage: 220,
        current: 5,
        enabled: true,
    };

    let device_info = ps.get_info(&device_name);

    assert_eq!(
        device_info,
        "PS 'socket1' specification:
                - type C
                - voltage=220
                - current=5
                - enabled=true
                - power=1100"
    );
}

#[test]
fn test_sensor_info() {
    let device_name = DeviceName("sensor1".to_string());

    let ts = TemperatureSensor {
        temperature: 26,
        range: SensorRange { min: 10, max: 40 },
        accuracy: 1,
    };

    let device_info = ts.get_info(&device_name);

    assert_eq!(
        device_info,
        "TS 'sensor1' specification:
                - 26C° 
                - range 10-40C° 
                - accuracy 1"
    );
}

#[test]
fn test_inventory_socket_info() {
    let room_name = RoomName("room1".to_string());
    let device_name = DeviceName("socket1".to_string());

    let power_sockets = HashMap::from([(
        room_name.clone(),
        HashMap::from([(
            device_name.clone(),
            DeviceItem::inject(PowerSocket {
                tpe: SocketType::C,
                voltage: 220,
                current: 5,
                enabled: true,
            }),
        )]),
    )]);

    let mdi = MemoryDeviceInventory::new(power_sockets);

    let device_info = mdi.get_info(&room_name, &device_name);

    assert_eq!(
        device_info.unwrap(),
        "PS 'socket1' specification:
                - type C
                - voltage=220
                - current=5
                - enabled=true
                - power=1100"
    );
}

#[test]
fn test_inventory_sensor_info() {
    let room_name = RoomName("room1".to_string());
    let device_name = DeviceName("sensor1".to_string());

    let temperature_sensors = HashMap::from([(
        room_name.clone(),
        HashMap::from([(
            device_name.clone(),
            DeviceItem::inject(TemperatureSensor {
                temperature: 26,
                range: SensorRange { min: 10, max: 40 },
                accuracy: 1,
            }),
        )]),
    )]);

    let mdi = MemoryDeviceInventory::new(temperature_sensors);

    let device_info = mdi.get_info(&room_name, &device_name);

    assert_eq!(
        device_info.unwrap(),
        "TS 'sensor1' specification:
                - 26C° 
                - range 10-40C° 
                - accuracy 1"
    );
}

#[test]
fn test_house_report() {
    let room_name = RoomName("room1".to_string());
    let socket_name = DeviceName("socket1".to_string());
    let sensor_name = DeviceName("sensor1".to_string());

    let power_sockets = HashMap::from([(
        socket_name.clone(),
        DeviceItem::inject(PowerSocket {
            tpe: SocketType::C,
            voltage: 220,
            current: 5,
            enabled: true,
        }),
    )]);

    let temperature_sensors = HashMap::from([(
        sensor_name.clone(),
        DeviceItem::inject(TemperatureSensor {
            temperature: 26,
            range: SensorRange { min: 10, max: 40 },
            accuracy: 1,
        }),
    )]);

    let devices = HashMap::from([(
        room_name.clone(),
        power_sockets
            .into_iter()
            .chain(temperature_sensors)
            .collect(),
    )]);

    let mdi = MemoryDeviceInventory::new(devices);

    let room = Room {
        name: room_name,
        devices: vec![socket_name, sensor_name],
    };

    let house = IntelligentHouse::from("house1", vec![room]);

    let report = house.generate_report(&mdi).unwrap();

    assert_eq!(
        report.trim(),
        "'house1' contains 1 rooms:
   'room1' has:
     PS 'socket1' specification:
                - type C
                - voltage=220
                - current=5
                - enabled=true
                - power=1100
     TS 'sensor1' specification:
                - 26C° 
                - range 10-40C° 
                - accuracy 1"
    );
}
