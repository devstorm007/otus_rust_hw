use threadpool::ThreadPool;

use house_server::domain::DeviceData::*;
use house_server::domain::RequestBody::{ChangeDeviceData, ShowDeviceInfo};
use house_server::domain::{DeviceLocation, RequestMessage};
use house_server::error::*;
use house_server::house_client::HouseClient;
use house_server::house_server::HouseServer;

fn main() -> Result<(), HouseExchangeError> {
    let pool: ThreadPool = ThreadPool::default();

    let tcp_server_address = "127.0.0.1:45932";

    let room_device_names = house::ThreeRoomNames::default();
    let inventory = house::mk_three_rooms_inventory(room_device_names);

    HouseServer::start(inventory, tcp_server_address, &pool)?;

    let mut client = HouseClient::connect(tcp_server_address, &pool)?;

    let response = client.send(RequestMessage {
        body: ShowDeviceInfo {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
        },
    })?;
    println!("kitchen->socket4 before disable: {:?}'", response.body);

    let response = client.send(RequestMessage {
        body: ChangeDeviceData {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
            data: PowerSocketState { enabled: false },
        },
    })?;
    println!("kitchen->socket4 try to disable: {:?}", response.body);

    let response = client.send(RequestMessage {
        body: ShowDeviceInfo {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
        },
    })?;
    println!("kitchen->socket4 after disable: {:?}", response.body);

    let response = client.send(RequestMessage {
        body: ChangeDeviceData {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
            data: PowerSocketState { enabled: true },
        },
    })?;
    println!("kitchen->socket4 try to enable: {:?}", response.body);

    let response = client.send(RequestMessage {
        body: ShowDeviceInfo {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
        },
    })?;
    println!("kitchen->socket4 after enable: {:?}", response.body);

    pool.join();

    Ok(())
}
