use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{channel, Receiver, Sender};

use flexbuffers::{DeserializationError, Reader, SerializationError};
use serde::{Deserialize, Serialize};
use threadpool::ThreadPool;

use exchange_protocol::domain::Message;
use tcp_exchange::tcp_client::TcpClient;
use udp_exchange::udp_client::UdpClient;

use crate::domain::{RequestBody, RequestMessage, ResponseMessage};
use crate::error::HouseExchangeError;
use crate::error::HouseExchangeError::*;

pub struct HouseClient {
    pub client_name: String,
    pub response_message_rx: Receiver<ResponseMessage>,
    request_message_tx: Sender<RequestMessage>,
}

impl HouseClient {
    pub fn connect<Addrs: ToSocketAddrs>(
        client_name: String,
        tcp_server_address: Addrs,
        udp_server_address: Addrs,
        local_address: Addrs,
        pool: &ThreadPool,
    ) -> Result<HouseClient, HouseExchangeError> {
        let tcp_client = TcpClient::connect(tcp_server_address, pool)?;
        let udp_client = UdpClient::connect(udp_server_address, local_address, pool)?;

        let (request_message_tx, request_message_rx) = channel::<RequestMessage>();

        let mut send_stream = tcp_client.clone_stream()?;
        let socket_clone = udp_client.clone_socket()?;
        let name = client_name.clone();
        pool.execute(move || {
            while let Ok(msg) = request_message_rx.recv() {
                let send_socket = socket_clone.try_clone().unwrap();
                Self::send_message(msg, &mut send_stream, send_socket, name.clone())
                    .unwrap_or_else(|error| {
                        eprintln!("client_{name}: send message to house server failed: {error:?}")
                    });
            }
        });

        let (response_message_tx, response_message_rx) = channel::<ResponseMessage>();

        Self::receive_messages(
            tcp_client.messages,
            tcp_client.server_address,
            response_message_tx.clone(),
            pool,
            client_name.clone(),
        );
        Self::receive_messages(
            udp_client.messages,
            udp_client.server_address,
            response_message_tx,
            pool,
            client_name.clone(),
        );

        Ok(HouseClient {
            client_name,
            response_message_rx,
            request_message_tx,
        })
    }

    fn receive_messages(
        receiver: Receiver<Message>,
        server_address: SocketAddr,
        response_message_tx: Sender<ResponseMessage>,
        pool: &ThreadPool,
        _client_name: String,
    ) {
        pool.execute(move || {
            while let Ok(msg) = receiver.recv() {
                match msg {
                    Message::Connected => {
                        println!("client_{_client_name}: connected to server '{}'", server_address)
                    }
                    Message::Bytes(ref response_bytes) => {
                        Self::receive_message(
                            response_bytes,
                            response_message_tx.clone(),
                            server_address,
                        ).unwrap_or_else(|error| {
                            eprintln!(
                                "client_{_client_name}: receiving message from house server '{server_address}' failed: {error:?}"
                            )
                        });
                    }
                    Message::Disconnected => {
                        println!("client_{_client_name}: disconnected from '{}'", server_address)
                    }
                };
            }
        });
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
        send_socket: UdpSocket,
        _client_name: String,
    ) -> Result<(), HouseExchangeError> {
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        request_message
            .serialize(&mut serializer)
            .map_err(|e| SerializationError::Serde(e.to_string()))?;

        let bytes = serializer.view();

        println!(
            "client_{_client_name}: send message {:?}",
            request_message.body
        );

        match request_message.body {
            RequestBody::ChangeDeviceData { .. } | RequestBody::ShowDeviceInfo { .. } => {
                let server_address = send_stream.peer_addr()?;
                println!("client_{_client_name}: send bytes by stream");
                TcpClient::send_by(send_stream, bytes)
                    .map_err(|e| SendNotifyError(server_address, e.to_string()))
            }
            RequestBody::RegisterDeviceMonitor { .. } | RequestBody::RemoveDeviceMonitor { .. } => {
                let server_address = send_socket.peer_addr()?;
                UdpClient::send_by(send_socket, server_address, bytes)
                    .map_err(|e| SendNotifyError(server_address, e.to_string()))
            }
        }
    }

    pub fn send_and_receive(
        &mut self,
        msg: RequestMessage,
    ) -> Result<ResponseMessage, HouseExchangeError> {
        println!(
            "client_{}: send message {msg:?} to bus",
            self.client_name.clone()
        );
        self.request_message_tx.send(msg).map_err(|e| {
            eprintln!("client: failed send to bus: {}", e);
            SendNotifyEventError(e.to_string())
        })?;

        println!("client_{}: wait response", self.client_name.clone());
        self.response_message_rx.recv().map_err(|_| ReceiveError)
    }

    pub fn send(&mut self, msg: RequestMessage) -> Result<(), HouseExchangeError> {
        println!(
            "client_{}: send message {msg:?} to bus",
            self.client_name.clone()
        );
        self.request_message_tx
            .send(msg)
            .map_err(|e| SendNotifyEventError(e.to_string()))
    }
}
