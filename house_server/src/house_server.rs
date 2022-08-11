use std::cell::RefCell;
use std::net::{SocketAddr, ToSocketAddrs};
use std::rc::Rc;

use flexbuffers::{DeserializationError, Reader, SerializationError};
use frunk::hlist;
use serde::{Deserialize, Serialize};
use threadpool::ThreadPool;

use house::devices::power_socket::PowerSocket;
use house::errors::intelligent_house_error::IntelligentHouseError;
use house::errors::intelligent_house_error::InventoryError;
use house::house::intelligent_house::{DeviceName, RoomName};
use house::inventory::device_inventory::DeviceInventory;
use house::inventory::memory_device_inventory::DeviceItem;
use tcp_exchange::domain::Message;
use tcp_exchange::tcp_server::TcpServer;

use crate::domain::RequestBody::*;
use crate::domain::ResponseBody::{DeviceDataChanged, DeviceDescription};
use crate::domain::{DeviceData, RequestBody, RequestMessage, ResponseMessage};
use crate::error::HouseExchangeError;

pub struct HouseServer {
  pub address: SocketAddr,
}

impl HouseServer {
  pub fn start<Addrs: ToSocketAddrs>(
    device_inventory: impl DeviceInventory + Send + Sync + 'static,
    address: Addrs,
    pool: &ThreadPool,
  ) -> Result<HouseServer, HouseExchangeError> {
    let tcp_server = TcpServer::start(address, pool)?;
    let house_server = HouseServer {
      address: tcp_server.address,
    };

    pool.execute(move || {
      let inventory = Rc::new(RefCell::new(device_inventory));

      while let Ok(notify) = tcp_server.messages.recv() {
        match notify.message {
          Message::Connected => println!("house server: client {} connected", notify.address),
          Message::Bytes(ref request_bytes) => {
            HouseServer::process(request_bytes, inventory.clone()).map_or_else(
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
          Message::Disconnected => println!("house server: client {} disconnected", notify.address),
        }
      }
    });

    Ok(house_server)
  }

  fn process(
    bytes: &Vec<u8>,
    inventory: Rc<RefCell<impl DeviceInventory>>,
  ) -> Result<Vec<u8>, HouseExchangeError> {
    let msg_reader = Reader::get_root(bytes.as_slice()).map_err(DeserializationError::Reader)?;
    let request = RequestMessage::deserialize(msg_reader)?;

    let response = HouseServer::process_request(request.body, inventory)?;

    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    response
      .serialize(&mut serializer)
      .map_err(|e| SerializationError::Serde(e.to_string()))?;

    Ok(serializer.take_buffer())
  }

  fn process_request(
    request_body: RequestBody,
    inventory: Rc<RefCell<impl DeviceInventory>>,
  ) -> Result<ResponseMessage, HouseExchangeError> {
    match request_body {
      ChangeDeviceData { location, data } => {
        let device_name = &DeviceName(location.device_name);
        let room_name = &RoomName(location.room_name);
        (*inventory)
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
        let info = (*inventory)
          .borrow()
          .get_info(
            &RoomName(location.room_name.clone()),
            &DeviceName(location.device_name),
          )
          .map_err(IntelligentHouseError::InventoryError)?;

        Ok(ResponseMessage {
          body: DeviceDescription(info),
        })
      }
    }
  }
}
