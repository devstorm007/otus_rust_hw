use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::{ToSocketAddrs, UdpSocket};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};

use exchange_protocol::codecs::MAX_SIZE;
use exchange_protocol::domain::Message;
use exchange_protocol::error::ExchangeError;
use exchange_protocol::error::ExchangeError::{Io, SendNotifyError};

pub struct UdpClient {
    pub address: SocketAddr,
    pub server_address: SocketAddr,
    pub messages: Arc<Mutex<Receiver<Message>>>,
    socket: Arc<UdpSocket>,
}

impl UdpClient {
    pub async fn connect<Addrs: ToSocketAddrs>(
        server_addrs: Addrs,
        local_address: Addrs,
    ) -> Result<UdpClient, ExchangeError> {
        let socket = UdpSocket::bind(local_address).await?;
        socket.connect(server_addrs).await?;

        let server_address = socket.peer_addr()?;
        let client_address = socket.local_addr()?;

        let socket_arc = Arc::new(socket);

        let (message_notifier_tx, message_notifier_rx) = mpsc::channel::<Message>(1000);
        let receive_socket = socket_arc.clone();
        tokio::spawn(async move {
            UdpClient::receive_messages(receive_socket, message_notifier_tx, server_address)
                .await
                .unwrap_or_else(|error| {
                    eprintln!("receiving messages failed from server '{server_address}': {error:?}")
                })
        });

        Ok(UdpClient {
            address: client_address,
            server_address,
            messages: Arc::new(Mutex::new(message_notifier_rx)),
            socket: socket_arc,
        })
    }

    async fn receive_messages(
        socket: Arc<UdpSocket>,
        message_notifier_tx: Sender<Message>,
        server_address: SocketAddr,
    ) -> Result<(), ExchangeError> {
        let mut buf = [0; MAX_SIZE];

        message_notifier_tx
            .send(Message::Connected)
            .await
            .map_err(|e| SendNotifyError(server_address, e.to_string()))?;

        while let Ok((received, server_address)) = socket.recv_from(&mut buf).await {
            message_notifier_tx
                .send(Message::Bytes(buf[..received].into()))
                .await
                .map_err(|e| SendNotifyError(server_address, e.to_string()))?;
        }

        Ok(())
    }

    pub async fn send(&mut self, bytes: &[u8]) -> Result<(), ExchangeError> {
        self.socket.send(bytes).await.map(|_| ()).map_err(Io)
    }

    /*pub async fn send(&self, bytes: &[u8]) -> io::Result<()> {
        Self::send_by(self.socket.try_clone()?, self.server_address, bytes).await
    }*/

    /*pub async fn send_by(
        socket: UdpSocket,
        server_address: SocketAddr,
        bytes: &[u8],
    ) -> io::Result<()> {
        socket
            .send_to(bytes, server_address)
            .await
            .unwrap_or_else(|error| {
                eprintln!(
                    "sending message to server '{}' failed: {error}",
                    server_address
                );
                0
            });
        Ok(())
    }*/

    /*pub fn clone_socket(&self) -> io::Result<UdpSocket> {
        self.socket.try_clone()
    }*/
}
