use house::devices::power_socket::{PowerSocket, SocketType};
use house::devices::temperature_sensor::{SensorRange, TemperatureSensor};
use house::house::domain::{DeviceName, Room, RoomName};
use house::inventory::domain::RoomDevices;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use web::error::HouseApiError;
use web::house_api::HouseAPI;

#[actix_web::main]
async fn main() -> Result<(), HouseApiError> {
    let server_address = "127.0.0.1:8089";
    let db_connection = "mongodb://root:example@localhost:27017".to_string();
    let drop_db = true;
    HouseAPI::start(server_address, db_connection, drop_db).await?;

    let client = Client::new();
    while !client
        .get(format!("http://{server_address}/readiness"))
        .send()
        .await
        .map(|resp| resp.status().is_success())
        .unwrap_or(false)
    {
        sleep(Duration::from_millis(100)).await;
    }

    let kitchen = RoomName("kitchen".to_string());
    let socket1 = DeviceName("socket1".to_string());
    let sensor1 = DeviceName("sensor1".to_string());

    add_rooms(server_address, &client, &kitchen).await?;
    add_inventory_devices(server_address, &client, &kitchen, &socket1, &sensor1).await?;
    add_house_devices(server_address, &client, &kitchen, &socket1, &sensor1).await?;

    let report = client
        .get(format!("http://{server_address}/report"))
        .send()
        .await?
        .json::<String>()
        .await?;

    assert_eq!(
        report.trim(),
        "'Plaza house' contains 1 rooms:
   'kitchen' has:
     PS 'socket1' specification:
                - type C
                - voltage=220
                - current=10
                - enabled=true
                - power=2200
     TS 'sensor1' specification:
                - 26C° 
                - range 10-40C° 
                - accuracy 1"
            .to_string()
    );

    Ok(())
}

async fn add_rooms(
    server_address: &str,
    client: &Client,
    kitchen: &RoomName,
) -> Result<(), HouseApiError> {
    client
        .post(format!("http://{server_address}/rooms/{}", kitchen.0))
        .send()
        .await?;
    let rooms = reqwest::get(format!("http://{server_address}/rooms"))
        .await?
        .json::<Vec<Room>>()
        .await?;
    assert_eq!(
        rooms,
        vec![Room {
            name: kitchen.clone(),
            devices: vec![],
        }]
    );

    let dining_room = RoomName("dining room".to_string());
    client
        .post(format!("http://{server_address}/rooms/{}", dining_room.0))
        .send()
        .await?;
    let rooms = reqwest::get(format!("http://{server_address}/rooms"))
        .await?
        .json::<Vec<Room>>()
        .await?;
    assert_eq!(
        rooms,
        vec![
            Room {
                name: kitchen.clone(),
                devices: vec![],
            },
            Room {
                name: dining_room.clone(),
                devices: vec![],
            }
        ]
    );

    client
        .delete(format!("http://{server_address}/rooms/{}", dining_room.0))
        .send()
        .await?;
    let rooms = reqwest::get(format!("http://{server_address}/rooms"))
        .await?
        .json::<Vec<Room>>()
        .await?;
    assert_eq!(
        rooms,
        vec![Room {
            name: kitchen.clone(),
            devices: vec![],
        }]
    );

    Ok(())
}

async fn add_inventory_devices(
    server_address: &str,
    client: &Client,
    kitchen: &RoomName,
    socket1: &DeviceName,
    sensor1: &DeviceName,
) -> Result<(), HouseApiError> {
    let power_socket = PowerSocket {
        tpe: SocketType::C,
        voltage: 220,
        current: 10,
        enabled: true,
    };
    client
        .post(format!(
            "http://{server_address}/inventory/{}/devices/socket/{}",
            kitchen.0, socket1.0
        ))
        .json(&power_socket)
        .send()
        .await?;

    let temperature_sensor = TemperatureSensor {
        temperature: 26,
        range: SensorRange { min: 10, max: 40 },
        accuracy: 1,
    };
    client
        .post(format!(
            "http://{server_address}/inventory/{}/devices/sensor/{}",
            kitchen.0, sensor1.0
        ))
        .json(&temperature_sensor)
        .send()
        .await?;

    let inventory_devices = client
        .get(format!("http://{server_address}/inventory"))
        .send()
        .await?
        .json::<Vec<RoomDevices>>()
        .await?;
    assert_eq!(
        inventory_devices,
        vec![RoomDevices {
            name: kitchen.clone(),
            sockets: HashMap::from([(socket1.clone(), power_socket)]),
            sensors: HashMap::from([(sensor1.clone(), temperature_sensor)])
        }]
    );

    Ok(())
}

async fn add_house_devices(
    server_address: &str,
    client: &Client,
    kitchen: &RoomName,
    socket1: &DeviceName,
    sensor1: &DeviceName,
) -> Result<(), HouseApiError> {
    client
        .post(format!(
            "http://{server_address}/rooms/{}/devices/{}",
            kitchen.0, socket1.0
        ))
        .send()
        .await?;
    client
        .post(format!(
            "http://{server_address}/rooms/{}/devices/{}",
            kitchen.0, sensor1.0
        ))
        .send()
        .await?;

    let room_devices = client
        .get(format!(
            "http://{server_address}/rooms/{}/devices",
            kitchen.0
        ))
        .send()
        .await?
        .json::<Vec<DeviceName>>()
        .await?;
    assert_eq!(room_devices, vec![socket1.clone(), sensor1.clone()]);

    Ok(())
}
