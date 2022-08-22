use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::mpsc::{channel, Receiver, Sender};

use threadpool::ThreadPool;

use exchange_protocol::codecs::MAX_SIZE;
use exchange_protocol::domain::{Message, NotifyMessage, SendMessage};
use exchange_protocol::error::ExchangeError;
use exchange_protocol::error::ExchangeError::SendNotifyError;

pub struct UdpServer {
    pub address: SocketAddr,
    pub messages: Receiver<NotifyMessage>,
    pub socket: UdpSocket,
}

impl UdpServer {
    pub fn start<Addrs: ToSocketAddrs>(
        address: Addrs,
        pool: &ThreadPool,
    ) -> Result<UdpServer, ExchangeError> {
        let socket = UdpSocket::bind(address)?;
        let server_address = socket.local_addr()?;

        let send_socket = socket.try_clone()?;
        let (client_sender_tx, client_sender_rx) = channel::<SendMessage>();
        pool.execute(move || {
            Self::send_messages(send_socket, client_sender_rx);
        });

        let (message_notifier_tx, message_notifier_rx) = channel::<NotifyMessage>();
        let receive_socket = socket.try_clone()?;
        pool.execute(move || {
            Self::receive_messages(receive_socket, client_sender_tx, message_notifier_tx)
                .unwrap_or_else(|error| eprintln!("receiving messages failed: {error:?}"))
        });

        println!("udp server started at {}", server_address.clone());

        Ok(UdpServer {
            address: server_address,
            messages: message_notifier_rx,
            socket,
        })
    }

    fn send_messages(socket: UdpSocket, client_sender_rx: Receiver<SendMessage>) {
        while let Ok(msg) = client_sender_rx.recv() {
            socket
                .send_to(msg.bytes.as_slice(), msg.client_address)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "sending message to client '{}' failed: {error}",
                        msg.client_address
                    );
                    0
                });
        }
    }

    fn receive_messages(
        socket: UdpSocket,
        client_sender_tx: Sender<SendMessage>,
        message_notifier_tx: Sender<NotifyMessage>,
    ) -> Result<(), ExchangeError> {
        let mut buf = [0; MAX_SIZE];
        while let Ok((received, client_address)) = socket.recv_from(&mut buf) {
            let msg = NotifyMessage::new(
                Message::Bytes(buf[..received].into()),
                client_address,
                client_sender_tx.clone(),
            );
            message_notifier_tx
                .send(msg)
                .map_err(|e| SendNotifyError(client_address, e.to_string()))?;
        }

        Ok(())
    }

    pub fn send_by(socket: UdpSocket, client_address: &SocketAddr, bytes: &[u8]) -> io::Result<()> {
        socket
            .send_to(bytes, client_address)
            .unwrap_or_else(|error| {
                eprintln!("sending message to client '{client_address}' failed: {error}");
                0
            });
        Ok(())
    }
}
