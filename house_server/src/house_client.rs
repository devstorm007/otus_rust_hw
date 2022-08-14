use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};

use flexbuffers::{DeserializationError, Reader, SerializationError};
use serde::{Deserialize, Serialize};
use threadpool::ThreadPool;

use exchange_model::domain::Message;
use tcp_exchange::tcp_client::TcpClient;

use crate::domain::{RequestMessage, ResponseMessage};
use crate::error::HouseExchangeError;
use crate::error::HouseExchangeError::*;

pub struct HouseClient {
    pub server_address: SocketAddr,
    response_message_rx: Receiver<ResponseMessage>,
    request_message_tx: Sender<RequestMessage>,
}

impl HouseClient {
    pub fn connect<T: ToSocketAddrs>(
        address: T,
        pool: &ThreadPool,
    ) -> Result<HouseClient, HouseExchangeError> {
        let tcp_client = TcpClient::connect(address, pool)?;
        let server_address = tcp_client.server_address;

        let (request_message_tx, request_message_rx) = channel::<RequestMessage>();

        let mut send_stream = tcp_client.clone_stream()?;
        pool.execute(move || {
      while let Ok(msg) = request_message_rx.recv() {
        HouseClient::send_message(msg, &mut send_stream, server_address).unwrap_or_else(|error| {
          eprintln!("client: send message to house server '{server_address}' failed: {error:?}")
        });
      }
    });

        let (response_message_tx, response_message_rx) = channel::<ResponseMessage>();

        pool.execute(move || {
            while let Ok(msg) = tcp_client.messages.recv() {
                match msg {
                    Message::Connected => {
                        println!("client: connected to server '{}'", server_address)
                    }
                    Message::Bytes(ref response_bytes) => {
                        HouseClient::receive_message(
                            response_bytes,
                            response_message_tx.clone(),
                            server_address,
                        )
                        .unwrap_or_else(|error| {
                            eprintln!(
                "client: receiving message from house server '{server_address}' failed: {error:?}"
              )
                        });
                    }
                    Message::Disconnected => {
                        println!("client: disconnected from '{}'", server_address)
                    }
                };
            }
        });

        Ok(HouseClient {
            server_address,
            response_message_rx,
            request_message_tx,
        })
    }

    fn receive_message(
        response_bytes: &Vec<u8>,
        response_message_tx: Sender<ResponseMessage>,
        server_address: SocketAddr,
    ) -> Result<(), HouseExchangeError> {
        let reader =
            Reader::get_root(response_bytes.as_slice()).map_err(DeserializationError::Reader)?;

        let response = ResponseMessage::deserialize(reader).unwrap();

        response_message_tx
            .send(response)
            .map_err(|e| SendNotifyError(server_address, e.to_string()))
    }

    fn send_message(
        request_message: RequestMessage,
        send_stream: &mut TcpStream,
        server_address: SocketAddr,
    ) -> Result<(), HouseExchangeError> {
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        request_message
            .serialize(&mut serializer)
            .map_err(|e| SerializationError::Serde(e.to_string()))?;

        let bytes = serializer.view();
        TcpClient::send_by_stream(send_stream, bytes)
            .map_err(|e| SendNotifyError(server_address, e.to_string()))
    }

    pub fn send(&mut self, msg: RequestMessage) -> Result<ResponseMessage, HouseExchangeError> {
        self.request_message_tx
            .send(msg)
            .map_err(|e| SendNotifyError(self.server_address, e.to_string()))?;

        self.response_message_rx.recv().map_err(|_| ReceiveError)
    }
}
