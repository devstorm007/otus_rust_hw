use std::cell::RefCell;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use dashmap::DashMap;
use flexbuffers::{DeserializationError, Reader, SerializationError};
use frunk::hlist;
use serde::{Deserialize, Serialize};
use threadpool::ThreadPool;

use exchange_protocol::domain::{Message, NotifyMessage};
use house::devices::power_socket::PowerSocket;
use house::devices::temperature_sensor::TemperatureSensor;
use house::errors::intelligent_house_error::IntelligentHouseError;
use house::errors::intelligent_house_error::InventoryError;
use house::house::intelligent_house::{DeviceName, RoomName};
use house::inventory::device_inventory::DeviceInventory;
use house::inventory::memory_device_inventory::DeviceItem;
use tcp_exchange::tcp_server::TcpServer;
use udp_exchange::udp_server::UdpServer;

use crate::domain::RequestBody::*;
use crate::domain::ResponseBody::*;
use crate::domain::{DeviceData, DeviceLocation, RequestBody, RequestMessage, ResponseMessage};
use crate::error::HouseExchangeError;

#[derive(Clone)]
pub struct HouseServer {
    pub tcp_address: SocketAddr,
    pub udp_address: SocketAddr,
}

impl HouseServer {
    pub fn start<Addrs: ToSocketAddrs>(
        device_inventory: impl DeviceInventory + Send + Sync + Clone + 'static,
        tcp_address: Addrs,
        udp_address: Addrs,
        pool: &ThreadPool,
    ) -> Result<HouseServer, HouseExchangeError> {
        let tcp_server = TcpServer::start(tcp_address, pool)?;
        let udp_server = UdpServer::start(udp_address, pool)?;

        let device_monitors = Arc::new(DashMap::<SocketAddr, DeviceLocation>::new());

        let house_server = HouseServer {
            tcp_address: tcp_server.address,
            udp_address: udp_server.address,
        };

        Self::process_exchange(
            tcp_server.messages,
            device_inventory.clone(),
            pool,
            device_monitors.clone(),
        );

        Self::process_exchange(
            udp_server.messages,
            device_inventory.clone(),
            pool,
            device_monitors.clone(),
        );

        Self::broadcast_monitors(device_monitors, device_inventory, udp_server.socket, pool);

        Ok(house_server)
    }

    fn broadcast_monitors(
        device_monitors: Arc<DashMap<SocketAddr, DeviceLocation>>,
        device_inventory: impl DeviceInventory + Clone + Send + Sync + 'static,
        udp_socket: UdpSocket,
        pool: &ThreadPool,
    ) {
        pool.execute(move || loop {
            device_monitors.iter().for_each(|r| {
                let (client_address, location): (&SocketAddr, &DeviceLocation) = r.pair();

                let data = Self::get_device_data(location, device_inventory.clone()).unwrap();

                UdpServer::send_by(
                    udp_socket.try_clone().unwrap(),
                    client_address,
                    data.as_slice(),
                )
                .unwrap()
            });
            thread::sleep(Duration::from_millis(500));
        });
    }

    fn get_device_data(
        location: &DeviceLocation,
        device_inventory: impl DeviceInventory,
    ) -> Result<Vec<u8>, HouseExchangeError> {
        let device = device_inventory
            .get_device(
                &RoomName(location.room_name.clone()),
                &DeviceName(location.device_name.clone()),
            )
            .map_err(IntelligentHouseError::InventoryError)?;

        let body = device.fold(hlist![
            |ps: PowerSocket| {
                PowerSocketInfo {
                    enabled: ps.enabled,
                    power: ps.power(),
                }
            },
            |ts: TemperatureSensor| {
                TemperatureSensorInfo {
                    temperature: ts.current_temperature(),
                }
            }
        ]);

        Self::serialize_response(ResponseMessage { body })
    }

    fn process_exchange(
        receiver: Receiver<NotifyMessage>,
        device_inventory: impl DeviceInventory + Clone + Send + Sync + 'static,
        pool: &ThreadPool,
        device_monitors: Arc<DashMap<SocketAddr, DeviceLocation>>,
    ) {
        pool.execute(move || {
            let inventory = RefCell::new(device_inventory);
            while let Ok(notify) = receiver.recv() {
                match notify.message {
                    Message::Connected => {
                        println!("house server: client {} connected", notify.address)
                    }
                    Message::Bytes(ref request_bytes) => {
                        Self::process_bytes(request_bytes, inventory.clone(), device_monitors.clone(), notify.address).map_or_else(
                            |error| {
                                eprintln!(
                                    "house server: process message from client '{}' failed: {error:?}",
                                    notify.address
                                );
                            },
                            |response_bytes| {
                                notify.reply(response_bytes).unwrap_or_else(|error| {
                                    eprintln!(
                                        "house server: send message to client '{}' failed: {error:?}",
                                        notify.address
                                    );
                                });
                            },
                        );
                    }
                    Message::Disconnected => {
                        println!("house server: client {} disconnected", notify.address)
                    }
                }
            }
        });
    }

    fn process_bytes(
        bytes: &Vec<u8>,
        inventory: RefCell<impl DeviceInventory>,
        device_monitors: Arc<DashMap<SocketAddr, DeviceLocation>>,
        sender_address: SocketAddr,
    ) -> Result<Vec<u8>, HouseExchangeError> {
        let msg_reader =
            Reader::get_root(bytes.as_slice()).map_err(DeserializationError::Reader)?;

        let request = RequestMessage::deserialize(msg_reader)?;

        let response =
            Self::process_request(request.body, inventory, device_monitors, sender_address)?;

        Self::serialize_response(response)
    }

    fn serialize_response(response: ResponseMessage) -> Result<Vec<u8>, HouseExchangeError> {
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        response
            .serialize(&mut serializer)
            .map_err(|e| SerializationError::Serde(e.to_string()))?;

        Ok(serializer.take_buffer())
    }

    fn process_request(
        request_body: RequestBody,
        inventory: RefCell<impl DeviceInventory>,
        device_monitors: Arc<DashMap<SocketAddr, DeviceLocation>>,
        sender_address: SocketAddr,
    ) -> Result<ResponseMessage, HouseExchangeError> {
        match request_body {
            ChangeDeviceData { location, data } => {
                let device_name = &DeviceName(location.device_name);
                let room_name = &RoomName(location.room_name);
                inventory
                    .borrow_mut()
                    .change_device(room_name, device_name, |device| match data {
                        DeviceData::PowerSocketState { enabled } => device.fold(hlist![
                            |mut ps: PowerSocket| {
                                ps.enabled = enabled;
                                Ok(DeviceItem::inject(ps))
                            },
                            |_| Err(InventoryError::InventoryDeviceInvalid(
                                device_name.clone(),
                                room_name.clone()
                            ))
                        ]),
                    })
                    .map_err(IntelligentHouseError::InventoryError)?;

                Ok(ResponseMessage {
                    body: DeviceDataChanged,
                })
            }
            ShowDeviceInfo { location } => {
                let info = inventory
                    .borrow()
                    .get_info(
                        &RoomName(location.room_name),
                        &DeviceName(location.device_name),
                    )
                    .map_err(IntelligentHouseError::InventoryError)?;

                Ok(ResponseMessage {
                    body: DeviceDescription(info),
                })
            }
            RegisterDeviceMonitor { location } => {
                device_monitors.insert(sender_address, location);
                Ok(ResponseMessage {
                    body: MonitorRegistered,
                })
            }
            RemoveDeviceMonitor => {
                device_monitors.remove(&sender_address);
                Ok(ResponseMessage {
                    body: MonitorRemoved,
                })
            }
        }
    }
}
