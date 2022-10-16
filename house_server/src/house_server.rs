use std::borrow::BorrowMut;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use flexbuffers::{DeserializationError, Reader, SerializationError};
use frunk::hlist;
use serde::{Deserialize, Serialize};
use tokio::net::ToSocketAddrs;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio::time::sleep;

use exchange_protocol::domain::{Message, NotifyMessage};
use house::devices::power_socket::PowerSocket;
use house::devices::temperature_sensor::TemperatureSensor;
use house::errors::intelligent_house_error::IntelligentHouseError;
use house::errors::intelligent_house_error::InventoryError;
use house::house::domain::*;
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
    pub async fn start<Addrs: ToSocketAddrs>(
        device_inventory: impl DeviceInventory + Send + Sync + Clone + 'static,
        tcp_address: Addrs,
        udp_address: Addrs,
    ) -> Result<HouseServer, HouseExchangeError> {
        let tcp_server = TcpServer::start(tcp_address).await?;
        let udp_server = UdpServer::start(udp_address).await?;

        let house_server = HouseServer {
            tcp_address: tcp_server.address,
            udp_address: udp_server.address,
        };

        let udp_server = Arc::new(Mutex::new(udp_server));
        let device_monitors = Arc::new(DashMap::<SocketAddr, DeviceLocation>::new());

        let inventory = device_inventory.clone();
        let monitors = device_monitors.clone();
        tokio::spawn(async move {
            let messages = tcp_server.messages.clone();
            Self::process_exchange(messages, inventory, monitors).await;
        });

        let inventory = device_inventory.clone();
        let monitors = device_monitors.clone();
        let server = udp_server.clone();
        tokio::spawn(async move {
            let messages = server.lock().await.messages.clone();
            Self::process_exchange(messages, inventory, monitors).await;
        });

        Self::broadcast_monitors(device_monitors, device_inventory, udp_server).await;

        Ok(house_server)
    }

    async fn process_exchange(
        messages: Arc<Mutex<Receiver<NotifyMessage>>>,
        device_inventory: impl DeviceInventory + Clone,
        device_monitors: Arc<DashMap<SocketAddr, DeviceLocation>>,
    ) {
        while let Some(notify) = messages.lock().await.recv().await {
            match notify.message {
                Message::Connected => {
                    println!("house server: client {} connected", notify.address)
                }
                Message::Bytes(ref request_bytes) => {
                    let result = Self::process_bytes(
                        request_bytes,
                        device_inventory.clone(),
                        device_monitors.clone(),
                        notify.address,
                    )
                    .await;

                    match result {
                        Ok(response_bytes) => {
                            notify.reply(response_bytes).await.unwrap_or_else(|error| {
                                eprintln!(
                                    "house server: send message to client '{}' failed: {error:?}",
                                    notify.address
                                );
                            })
                        }
                        Err(error) => eprintln!(
                            "house server: process message from client '{}' failed: {error:?}",
                            notify.address
                        ),
                    }
                }
                Message::Disconnected => {
                    println!("house server: client {} disconnected", notify.address)
                }
            }
        }
    }

    async fn process_bytes(
        bytes: &Vec<u8>,
        device_inventory: impl DeviceInventory,
        device_monitors: Arc<DashMap<SocketAddr, DeviceLocation>>,
        sender_address: SocketAddr,
    ) -> Result<Vec<u8>, HouseExchangeError> {
        let msg_reader =
            Reader::get_root(bytes.as_slice()).map_err(DeserializationError::Reader)?;

        let request = RequestMessage::deserialize(msg_reader)?;

        let response = Self::process_request(
            request.body,
            device_inventory,
            device_monitors,
            sender_address,
        )
        .await?;

        Self::serialize_response(response)
    }

    fn serialize_response(response: ResponseMessage) -> Result<Vec<u8>, HouseExchangeError> {
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        response
            .serialize(&mut serializer)
            .map_err(|e| SerializationError::Serde(e.to_string()))?;

        Ok(serializer.take_buffer())
    }

    async fn process_request(
        request_body: RequestBody,
        mut device_inventory: impl DeviceInventory,
        device_monitors: Arc<DashMap<SocketAddr, DeviceLocation>>,
        sender_address: SocketAddr,
    ) -> Result<ResponseMessage, HouseExchangeError> {
        match request_body {
            ChangeDeviceData { location, data } => {
                let device_name = &DeviceName(location.device_name);
                let room_name = &RoomName(location.room_name);
                device_inventory
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
                    .await
                    .map_err(IntelligentHouseError::InventoryError)?;

                Ok(ResponseMessage {
                    body: DeviceDataChanged,
                })
            }
            ShowDeviceInfo { location } => {
                let info = device_inventory
                    .get_info(
                        &RoomName(location.room_name),
                        &DeviceName(location.device_name),
                    )
                    .await
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

    async fn broadcast_monitors(
        device_monitors: Arc<DashMap<SocketAddr, DeviceLocation>>,
        device_inventory: impl DeviceInventory + Clone + Send + Sync + 'static,
        udp_server: Arc<Mutex<UdpServer>>,
    ) {
        tokio::spawn(async move {
            loop {
                for dm in device_monitors.iter() {
                    let (client_address, location) = dm.pair();

                    let data = Self::get_device_data(location, device_inventory.clone())
                        .await
                        .unwrap();
                    udp_server.lock().await
                      .send(client_address, data.as_slice())
                      .await
                      .unwrap_or_else(|error| {
                          eprintln!(
                              "house server: sending message to '{client_address}' failed: {error:?}"
                          )
                      });
                }
                sleep(Duration::from_millis(500)).await;
            }
        });
    }

    async fn get_device_data(
        location: &DeviceLocation,
        device_inventory: impl DeviceInventory,
    ) -> Result<Vec<u8>, HouseExchangeError> {
        let device = device_inventory
            .get_device(
                &RoomName(location.room_name.clone()),
                &DeviceName(location.device_name.clone()),
            )
            .await
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
}
