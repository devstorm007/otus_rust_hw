use std::io;
use std::io::Bytes;
use std::io::{Read, Write};
use std::net::{
    Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream, ToSocketAddrs, UdpSocket,
};
use std::sync::mpsc::{channel, Receiver, Sender};

use threadpool::ThreadPool;

use exchange_protocol::codecs::{decode_bytes, encode_bytes, MAX_SIZE};
use exchange_protocol::domain::{Message, NotifyMessage, SendMessage};
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
        local_addrs: Addrs,
        server_addrs: Addrs,
        pool: &ThreadPool,
    ) -> Result<UdpClient, ExchangeError> {
        let socket = UdpSocket::bind(local_addrs)?;
        socket.connect(server_addrs)?;

        let server_address = socket.peer_addr()?;
        let client_address = socket.local_addr()?;

        let (message_notifier_tx, message_notifier_rx) = channel::<Message>();
        let receive_socket = socket.try_clone()?;
        pool.execute(move || {
            UdpClient::receive_messages(receive_socket, message_notifier_tx).unwrap_or_else(
                |error| {
                    eprintln!("receiving messages failed from server '{server_address}': {error:?}")
                },
            )
        });

        println!("connected to server '{}'", server_address);

        Ok(UdpClient {
            address: client_address,
            server_address: server_address,
            messages: message_notifier_rx,
            socket,
        })
    }

    fn receive_messages(
        socket: UdpSocket,
        message_notifier_tx: Sender<Message>,
    ) -> Result<(), ExchangeError> {
        let mut buf = [0; MAX_SIZE];

        while let Ok((received, server_address)) = socket.recv_from(&mut buf) {
            message_notifier_tx
                .send(Message::Bytes(buf[..received].into()))
                .map_err(|e| SendNotifyError(server_address, e.to_string()))?;
        }

        Ok(())
    }

    pub fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.socket
            .send_to(bytes, self.server_address)
            .unwrap_or_else(|error| {
                eprintln!(
                    "sending message to server '{}' failed: {error}",
                    self.server_address
                );
                0
            });
        Ok(())
    }
}
