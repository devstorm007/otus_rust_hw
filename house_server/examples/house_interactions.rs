use std::time::Duration;

use house::inventory::memory_device_inventory::MemoryDeviceInventory;
use tokio::time::sleep;

use house_server::domain::DeviceData::*;
use house_server::domain::RequestBody::{
    ChangeDeviceData, RegisterDeviceMonitor, RemoveDeviceMonitor, ShowDeviceInfo,
};
use house_server::domain::ResponseBody::MonitorRemoved;
use house_server::domain::{DeviceLocation, RequestMessage, ResponseMessage};
use house_server::error::*;
use house_server::house_client::HouseClient;
use house_server::house_server::HouseServer;

#[tokio::main]
async fn main() -> Result<(), HouseExchangeError> {
    let tcp_server_address = "127.0.0.1:45932";
    let udp_server_address = "127.0.0.1:45959";

    let room_device_names = house::ThreeRoomNames::default();
    let inventory: MemoryDeviceInventory = house::mk_three_rooms_inventory(room_device_names);

    HouseServer::start(inventory, tcp_server_address, udp_server_address).await?;

    sleep(Duration::from_secs(2)).await;

    let mut client = HouseClient::connect(
        "first".to_string(),
        tcp_server_address,
        udp_server_address,
        "127.0.0.1:41858",
    )
    .await?;

    let response = client
        .send_and_receive(RequestMessage {
            body: ShowDeviceInfo {
                location: DeviceLocation {
                    room_name: "kitchen".to_string(),
                    device_name: "socket4".to_string(),
                },
            },
        })
        .await?;

    println!(
        "client_first: kitchen->socket4 before disable: {:?}'",
        response.body
    );

    let response = client
        .send_and_receive(RequestMessage {
            body: ChangeDeviceData {
                location: DeviceLocation {
                    room_name: "kitchen".to_string(),
                    device_name: "socket4".to_string(),
                },
                data: PowerSocketState { enabled: false },
            },
        })
        .await?;
    println!(
        "client_first: kitchen->socket4 try to disable: {:?}",
        response.body
    );

    let response = client
        .send_and_receive(RequestMessage {
            body: ShowDeviceInfo {
                location: DeviceLocation {
                    room_name: "kitchen".to_string(),
                    device_name: "socket4".to_string(),
                },
            },
        })
        .await?;
    println!(
        "client_first: kitchen->socket4 after disable: {:?}",
        response.body
    );

    let response = client
        .send_and_receive(RequestMessage {
            body: ChangeDeviceData {
                location: DeviceLocation {
                    room_name: "kitchen".to_string(),
                    device_name: "socket4".to_string(),
                },
                data: PowerSocketState { enabled: true },
            },
        })
        .await?;
    println!(
        "client_first: kitchen->socket4 try to enable: {:?}",
        response.body
    );

    let response = client
        .send_and_receive(RequestMessage {
            body: ShowDeviceInfo {
                location: DeviceLocation {
                    room_name: "kitchen".to_string(),
                    device_name: "socket4".to_string(),
                },
            },
        })
        .await?;
    println!(
        "client_first: kitchen->socket4 after enable: {:?}",
        response.body
    );

    let sensor_location = DeviceLocation {
        room_name: "kitchen".to_string(),
        device_name: "sensor1".to_string(),
    };
    client
        .send(RequestMessage {
            body: RegisterDeviceMonitor {
                location: sensor_location.clone(),
            },
        })
        .await?;

    let mut i = 0;
    while let Some(response) = client.response_message_rx.recv().await {
        match response {
            ResponseMessage {
                body: MonitorRemoved { .. },
            } => {
                println!("client_first: left monitoring {:?}", sensor_location);
                break;
            }
            ResponseMessage { body } => {
                println!("client_first: received {:?}", body);
            }
        }
        i += 1;
        if i == 5 {
            client
                .send(RequestMessage {
                    body: RemoveDeviceMonitor,
                })
                .await?;
        }
    }

    println!("Interactions are completed");

    Ok(())
}
