use std::net::SocketAddr;
use std::sync::Arc;

use flexbuffers::{DeserializationError, Reader, SerializationError};
use serde::{Deserialize, Serialize};
use tokio::net::ToSocketAddrs;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};

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
    pub async fn connect<Addrs: ToSocketAddrs>(
        client_name: String,
        tcp_server_address: Addrs,
        udp_server_address: Addrs,
        local_address: Addrs,
    ) -> Result<HouseClient, HouseExchangeError> {
        let client = TcpClient::connect(tcp_server_address).await?;
        let tcp_server_address = client.server_address;
        let tcp_client_orig = Arc::new(Mutex::new(client));

        let client = UdpClient::connect(udp_server_address, local_address).await?;
        let udp_server_address = client.server_address;
        let udp_client_orig = Arc::new(Mutex::new(client));

        let (request_message_tx, mut request_message_rx) = mpsc::channel::<RequestMessage>(1000);

        let tcp_client = tcp_client_orig.clone();
        let udp_client = udp_client_orig.clone();
        let name = client_name.clone();
        tokio::spawn(async move {
            while let Some(msg) = request_message_rx.recv().await {
                Self::send_message(msg, tcp_client.clone(), udp_client.clone(), name.clone())
                    .await
                    .unwrap_or_else(|error| {
                        eprintln!("client_{name}: send message to house server failed: {error:?}")
                    });
            }
        });

        let (response_message_tx, response_message_rx) = mpsc::channel::<ResponseMessage>(1000);

        let response_tx = response_message_tx.clone();
        let name = client_name.clone();
        tokio::spawn(async move {
            let messages = (tcp_client_orig.clone()).lock().await.messages.clone();
            Self::receive_messages(messages, tcp_server_address, response_tx, name).await;
        });

        let name = client_name.clone();
        tokio::spawn(async move {
            let messages = (udp_client_orig.clone()).lock().await.messages.clone();
            Self::receive_messages(messages, udp_server_address, response_message_tx, name).await;
        });

        Ok(HouseClient {
            client_name,
            response_message_rx,
            request_message_tx,
        })
    }

    async fn send_message(
        request_message: RequestMessage,
        tcp_client: Arc<Mutex<TcpClient>>,
        udp_client: Arc<Mutex<UdpClient>>,
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
                let mut client = tcp_client.lock().await;
                client
                    .send(bytes)
                    .await
                    .map_err(|e| SendNotifyError(client.server_address, e.to_string()))
            }
            RequestBody::RegisterDeviceMonitor { .. } | RequestBody::RemoveDeviceMonitor { .. } => {
                let mut client = udp_client.lock().await;
                client
                    .send(bytes)
                    .await
                    .map_err(|e| SendNotifyError(client.server_address, e.to_string()))
            }
        }
    }

    pub async fn send_and_receive(
        &mut self,
        msg: RequestMessage,
    ) -> Result<ResponseMessage, HouseExchangeError> {
        self.request_message_tx.send(msg).await.map_err(|e| {
            eprintln!("client: failed send to bus: {}", e);
            SendNotifyEventError(e.to_string())
        })?;
        self.response_message_rx.recv().await.ok_or(ReceiveError)
    }

    pub async fn send(&mut self, msg: RequestMessage) -> Result<(), HouseExchangeError> {
        self.request_message_tx
            .send(msg)
            .await
            .map_err(|e| SendNotifyEventError(e.to_string()))
    }

    async fn receive_messages(
        messages: Arc<Mutex<Receiver<Message>>>,
        server_address: SocketAddr,
        response_message_tx: Sender<ResponseMessage>,
        _client_name: String,
    ) {
        while let Some(msg) = messages.lock().await.recv().await {
            match msg {
                Message::Connected => {
                    println!(
                        "client_{_client_name}: connected to server '{}'",
                        server_address
                    )
                }
                Message::Bytes(ref response_bytes) => {
                    Self::receive_message(
                        response_bytes,
                        response_message_tx.clone(),
                        server_address,
                    ).await.unwrap_or_else(|error| {
                        eprintln!(
                            "client_{_client_name}: receiving message from house server '{server_address}' failed: {error:?}"
                        )
                    });
                }
                Message::Disconnected => {
                    println!(
                        "client_{_client_name}: disconnected from '{}'",
                        server_address
                    )
                }
            };
        }
    }

    async fn receive_message(
        response_bytes: &Vec<u8>,
        response_message_tx: Sender<ResponseMessage>,
        server_address: SocketAddr,
    ) -> Result<(), HouseExchangeError> {
        let reader =
            Reader::get_root(response_bytes.as_slice()).map_err(DeserializationError::Reader)?;

        let response = ResponseMessage::deserialize(reader).unwrap();

        response_message_tx
            .send(response)
            .await
            .map_err(|e| SendNotifyError(server_address, e.to_string()))
    }
}
