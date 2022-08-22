use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{channel, Receiver, Sender};

use threadpool::ThreadPool;

use exchange_protocol::codecs::MAX_SIZE;
use exchange_protocol::domain::Message;
use exchange_protocol::error::ExchangeError;
use exchange_protocol::error::ExchangeError::SendNotifyError;

pub struct UdpClient {
    pub address: SocketAddr,
    pub server_address: SocketAddr,
    pub messages: Receiver<Message>,
    socket: UdpSocket,
}

impl UdpClient {
    pub fn connect<Addrs: ToSocketAddrs>(
        server_addrs: Addrs,
        pool: &ThreadPool,
    ) -> Result<UdpClient, ExchangeError> {
        let local_address = "127.0.0.1:41858";
        let socket = UdpSocket::bind(local_address)?;
        socket.connect(server_addrs)?;

        let server_address = socket.peer_addr()?;
        let client_address = socket.local_addr()?;

        let (message_notifier_tx, message_notifier_rx) = channel::<Message>();
        let receive_socket = socket.try_clone()?;
        pool.execute(move || {
            UdpClient::receive_messages(receive_socket, message_notifier_tx, server_address)
                .unwrap_or_else(|error| {
                    eprintln!("receiving messages failed from server '{server_address}': {error:?}")
                })
        });

        println!("connected to server '{}'", server_address);

        Ok(UdpClient {
            address: client_address,
            server_address,
            messages: message_notifier_rx,
            socket,
        })
    }

    fn receive_messages(
        socket: UdpSocket,
        message_notifier_tx: Sender<Message>,
        server_address: SocketAddr,
    ) -> Result<(), ExchangeError> {
        let mut buf = [0; MAX_SIZE];

        message_notifier_tx
            .send(Message::Connected)
            .map_err(|e| SendNotifyError(server_address, e.to_string()))?;

        while let Ok((received, server_address)) = socket.recv_from(&mut buf) {
            message_notifier_tx
                .send(Message::Bytes(buf[..received].into()))
                .map_err(|e| SendNotifyError(server_address, e.to_string()))?;
        }

        Ok(())
    }

    pub fn send(&self, bytes: &[u8]) -> io::Result<()> {
        Self::send_by(self.socket.try_clone()?, self.server_address, bytes)
    }

    pub fn send_by(socket: UdpSocket, server_address: SocketAddr, bytes: &[u8]) -> io::Result<()> {
        socket
            .send_to(bytes, server_address)
            .unwrap_or_else(|error| {
                eprintln!(
                    "sending message to server '{}' failed: {error}",
                    server_address
                );
                0
            });
        Ok(())
    }

    pub fn clone_socket(&self) -> io::Result<UdpSocket> {
        self.socket.try_clone()
    }
}
