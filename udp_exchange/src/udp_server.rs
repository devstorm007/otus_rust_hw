use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::{ToSocketAddrs, UdpSocket};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use exchange_protocol::codecs::MAX_SIZE;
use exchange_protocol::domain::{Message, NotifyMessage, SendMessage};
use exchange_protocol::error::ExchangeError;
use exchange_protocol::error::ExchangeError::{Io, SendNotifyError};

pub struct UdpServer {
    pub address: SocketAddr,
    pub messages: Receiver<NotifyMessage>,
    pub socket: Arc<UdpSocket>,
}

impl UdpServer {
    pub async fn start<Addrs: ToSocketAddrs>(address: Addrs) -> Result<UdpServer, ExchangeError> {
        let socket = UdpSocket::bind(address).await?;
        let server_address = socket.local_addr()?;

        let socket_arc = Arc::new(socket);

        let (client_sender_tx, client_sender_rx) = mpsc::channel::<SendMessage>(1000);
        let socket_clone = socket_arc.clone();
        tokio::spawn(async move {
            Self::send_messages(socket_clone, client_sender_rx).await;
        });

        let (message_notifier_tx, message_notifier_rx) = mpsc::channel::<NotifyMessage>(1000);
        let socket_clone = socket_arc.clone();
        tokio::spawn(async move {
            Self::receive_messages(socket_clone, client_sender_tx, message_notifier_tx)
                .await
                .unwrap_or_else(|error| {
                    eprintln!("udp_server: receiving messages failed: {error:?}")
                })
        });

        println!("udp_server: started at {}", server_address.clone());

        Ok(UdpServer {
            address: server_address,
            messages: message_notifier_rx,
            socket: socket_arc,
        })
    }

    async fn send_messages(socket: Arc<UdpSocket>, mut client_sender_rx: Receiver<SendMessage>) {
        while let Some(msg) = client_sender_rx.recv().await {
            socket
                .send_to(msg.bytes.as_slice(), msg.client_address)
                .await
                .unwrap_or_else(|error| {
                    eprintln!(
                        "udp_server: sending message to client '{}' failed: {error}",
                        msg.client_address
                    );
                    0
                });
        }
    }

    async fn receive_messages(
        socket: Arc<UdpSocket>,
        client_sender_tx: Sender<SendMessage>,
        message_notifier_tx: Sender<NotifyMessage>,
    ) -> Result<(), ExchangeError> {
        let mut buf = [0; MAX_SIZE];

        while let Ok((received, client_address)) = socket.recv_from(&mut buf).await {
            message_notifier_tx
                .send(NotifyMessage::new(
                    Message::Bytes(buf[..received].into()),
                    client_address,
                    client_sender_tx.clone(),
                ))
                .await
                .map_err(|e| SendNotifyError(client_address, e.to_string()))?;
        }

        Ok(())
    }

    pub async fn send(
        &mut self,
        client_address: &SocketAddr,
        bytes: &[u8],
    ) -> Result<(), ExchangeError> {
        self.socket
            .send_to(bytes, client_address)
            .await
            .map(|_| ())
            .map_err(Io)
        /*.unwrap_or_else(|error| {
            eprintln!("sending message to client '{client_address}' failed: {error}");
        })*/
    }

    /*pub async fn send_by(
        socket: UdpSocket,
        client_address: &SocketAddr,
        bytes: &[u8],
    ) -> io::Result<()> {
        unreachable!()
        /*socket
            .send_to(bytes, client_address)
            .unwrap_or_else(|error| {
                eprintln!("sending message to client '{client_address}' failed: {error}");
                0
            });
        Ok(())*/
    }*/
}
