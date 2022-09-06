use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};

use exchange_protocol::codecs::{decode_bytes_async, encode_bytes};
use exchange_protocol::domain::*;
use exchange_protocol::error::ExchangeError;
use exchange_protocol::error::ExchangeError::SendNotifyError;

pub struct TcpClient {
    pub address: SocketAddr,
    pub server_address: SocketAddr,
    pub messages: Arc<Mutex<Receiver<Message>>>,
    send_writer: OwnedWriteHalf,
}

/*impl Clone for TcpClient {
    fn clone(&self) -> Self {
        TcpClient {
            address: self.address,
            server_address: self.server_address,
            messages: self.message_notifier_rx,
            send_writer: self.send_writer.clone(),
        }
    }
}*/

impl TcpClient {
    pub async fn connect<T: ToSocketAddrs>(address: T) -> Result<TcpClient, ExchangeError> {
        let stream = TcpStream::connect(address).await?;
        let server_address: SocketAddr = stream.peer_addr()?;
        let client_address = stream.local_addr()?;

        let (message_notifier_tx, message_notifier_rx) = mpsc::channel::<Message>(1000);

        let (reader, writer) = TcpStream::into_split(stream);
        tokio::spawn(async move {
            TcpClient::process_receiving_messages(reader, server_address, message_notifier_tx)
                .await
                .unwrap_or_else(|error| {
                    eprintln!(
                        "tcp_client: receiving messages failed from '{server_address}': {error:?}"
                    )
                });
        });

        Ok(TcpClient {
            address: client_address,
            server_address,
            messages: Arc::new(Mutex::new(message_notifier_rx)),
            send_writer: writer,
        })
    }

    async fn process_receiving_messages(
        mut reader: OwnedReadHalf,
        server_address: SocketAddr,
        message_notifier_tx: Sender<Message>,
    ) -> Result<(), ExchangeError> {
        message_notifier_tx
            .send(Message::Connected)
            .await
            .map_err(|e| SendNotifyError(server_address, e.to_string()))?;

        while let Ok(bytes) = decode_bytes_async(&mut reader).await {
            message_notifier_tx
                .send(Message::Bytes(bytes))
                .await
                .map_err(|e| SendNotifyError(server_address, e.to_string()))?;
        }

        message_notifier_tx
            .send(Message::Disconnected)
            .await
            .map_err(|e| SendNotifyError(server_address, e.to_string()))
    }

    pub async fn send(&mut self, bytes: &[u8]) -> Result<(), ExchangeError> {
        let encoded = encode_bytes(bytes);

        let server_address = self.send_writer.peer_addr()?;

        self.send_writer
            .write_all(&encoded)
            .await
            .unwrap_or_else(|error| {
                eprintln!(
                    "tcp_client: sending message to server '{server_address}' failed: {error}"
                )
            });

        Ok(())
    }
}
