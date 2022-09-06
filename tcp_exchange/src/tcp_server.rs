use std::net::SocketAddr;

use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use exchange_protocol::codecs::{decode_bytes_async, encode_bytes};
use exchange_protocol::domain::{Message, NotifyMessage, SendMessage};
use exchange_protocol::error::ExchangeError;
use exchange_protocol::error::ExchangeError::SendNotifyError;

pub struct TcpServer {
    pub address: SocketAddr,
    pub messages: Receiver<NotifyMessage>,
}

impl TcpServer {
    pub async fn start<Addrs: ToSocketAddrs>(address: Addrs) -> Result<TcpServer, ExchangeError> {
        let listener = TcpListener::bind(address).await?;
        let server_address = listener.local_addr()?;

        let (message_notifier_tx, message_notifier_rx) = mpsc::channel::<NotifyMessage>(1000);

        tokio::spawn(async move {
            loop {
                Self::start_stream_processing(&listener, message_notifier_tx.clone())
                    .await
                    .unwrap_or_else(|error| {
                        eprintln!("tcp_server: connection process failed: {error:?}")
                    });
            }
        });

        println!("tcp_server: started at {server_address}");

        Ok(TcpServer {
            address: server_address,
            messages: message_notifier_rx,
        })
    }

    async fn start_stream_processing(
        listener: &TcpListener,
        message_notifier_tx: Sender<NotifyMessage>,
    ) -> Result<(), ExchangeError> {
        let (client_stream, client_address) = listener.accept().await?;

        let (client_sender_tx, mut client_sender_rx) = mpsc::channel::<SendMessage>(1000);

        let (reader, mut writer) = TcpStream::into_split(client_stream);

        message_notifier_tx
            .send(NotifyMessage::new(
                Message::Connected,
                client_address,
                client_sender_tx.clone(),
            ))
            .await
            .map_err(|e| SendNotifyError(client_address, e.to_string()))?;

        tokio::spawn(async move {
            TcpServer::process_receiving_messages(
                reader,
                client_address,
                client_sender_tx.clone(),
                message_notifier_tx.clone(),
            )
            .await
            .unwrap_or_else(|error| {
                eprintln!("tcp_server: receiving messages failed for '{client_address}': {error:?}")
            });
        });

        tokio::spawn(async move {
            while let Some(SendMessage { bytes, .. }) = client_sender_rx.recv().await {
                let encoded = encode_bytes(bytes.as_slice());
                writer.write_all(&encoded).await.unwrap_or_else(|error| {
                    eprintln!(
                        "tcp_server: sending message to client '{client_address}' failed: {error}"
                    )
                });
            }
        });

        Ok(())
    }

    async fn process_receiving_messages(
        mut reader: OwnedReadHalf,
        client_address: SocketAddr,
        client_sender_tx: Sender<SendMessage>,
        message_notifier_tx: Sender<NotifyMessage>,
    ) -> Result<(), ExchangeError> {
        while let Ok(message) = decode_bytes_async(&mut reader).await {
            message_notifier_tx
                .send(NotifyMessage::new(
                    Message::Bytes(message),
                    client_address,
                    client_sender_tx.clone(),
                ))
                .await
                .map_err(|e| SendNotifyError(client_address, e.to_string()))?;
        }

        message_notifier_tx
            .send(NotifyMessage::new(
                Message::Disconnected,
                client_address,
                client_sender_tx,
            ))
            .await
            .map_err(|e| SendNotifyError(client_address, e.to_string()))
    }
}
