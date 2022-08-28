use threadpool::ThreadPool;

use house_server::domain::DeviceData::*;
use house_server::domain::RequestBody::{
    ChangeDeviceData, RegisterDeviceMonitor, RemoveDeviceMonitor, ShowDeviceInfo,
};
use house_server::domain::ResponseBody::MonitorRemoved;
use house_server::domain::{DeviceLocation, RequestMessage, ResponseMessage};
use house_server::error::*;
use house_server::house_client::HouseClient;

fn main() -> Result<(), HouseExchangeError> {
    let pool: ThreadPool = ThreadPool::default();

    let tcp_server_address = "127.0.0.1:45932";
    let udp_server_address = "127.0.0.1:45959";
    let udp_local_address = "127.0.0.1:41868";

    let mut client = HouseClient::connect(
        "b".to_string(),
        tcp_server_address,
        udp_server_address,
        udp_local_address,
        &pool,
    )?;

    let response = client.send_and_receive(RequestMessage {
        body: ShowDeviceInfo {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
        },
    })?;
    println!(
        "client_b: kitchen->socket4 before disable: {:?}'",
        response.body
    );

    let response = client.send_and_receive(RequestMessage {
        body: ChangeDeviceData {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
            data: PowerSocketState { enabled: false },
        },
    })?;
    println!(
        "client_b: kitchen->socket4 try to disable: {:?}",
        response.body
    );

    let response = client.send_and_receive(RequestMessage {
        body: ShowDeviceInfo {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
        },
    })?;
    println!(
        "client_b: kitchen->socket4 after disable: {:?}",
        response.body
    );

    let response = client.send_and_receive(RequestMessage {
        body: ChangeDeviceData {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
            data: PowerSocketState { enabled: true },
        },
    })?;
    println!(
        "client_b: kitchen->socket4 try to enable: {:?}",
        response.body
    );

    let response = client.send_and_receive(RequestMessage {
        body: ShowDeviceInfo {
            location: DeviceLocation {
                room_name: "kitchen".to_string(),
                device_name: "socket4".to_string(),
            },
        },
    })?;
    println!(
        "client_b: kitchen->socket4 after enable: {:?}",
        response.body
    );

    let sensor_location = DeviceLocation {
        room_name: "kitchen".to_string(),
        device_name: "sensor1".to_string(),
    };
    client.send(RequestMessage {
        body: RegisterDeviceMonitor {
            location: sensor_location.clone(),
        },
    })?;

    let mut i = 0;
    while let Ok(response) = client.response_message_rx.recv() {
        match response {
            ResponseMessage {
                body: MonitorRemoved { .. },
            } => {
                println!("client_b: left monitoring {:?}", sensor_location);
                break;
            }
            ResponseMessage { body } => {
                println!("client_b: received {:?}", body);
            }
        }
        i += 1;
        if i == 56 {
            client.send(RequestMessage {
                body: RemoveDeviceMonitor,
            })?;
        }
    }

    pool.join();

    Ok(())
}
