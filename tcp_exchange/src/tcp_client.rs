use std::io;
use std::io::Bytes;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};

use threadpool::ThreadPool;

use exchange_protocol::domain::*;
use exchange_protocol::error::ExchangeError;
use exchange_protocol::error::ExchangeError::SendNotifyError;

use exchange_protocol::codecs::{decode_bytes, encode_bytes};

pub struct TcpClient {
    pub address: SocketAddr,
    pub server_address: SocketAddr,
    pub messages: Receiver<Message>,
    send_stream: TcpStream,
}

impl TcpClient {
    pub fn connect<T: ToSocketAddrs>(
        address: T,
        pool: &ThreadPool,
    ) -> Result<TcpClient, ExchangeError> {
        let stream = TcpStream::connect(address)?;
        let server_address: SocketAddr = stream.peer_addr()?;
        let client_address = stream.local_addr()?;

        let (message_notifier_tx, message_notifier_rx) = channel();

        let send_stream = stream.try_clone()?;

        let receive_stream = stream.try_clone()?;
        pool.execute(move || {
            TcpClient::process_receiving_messages(
                receive_stream.bytes(),
                server_address,
                message_notifier_tx,
            )
            .unwrap_or_else(|error| {
                eprintln!("receiving messages failed for '{server_address}': {error:?}")
            });

            stream.shutdown(Shutdown::Both).unwrap_or_default();
        });

        Ok(TcpClient {
            address: client_address,
            server_address,
            messages: message_notifier_rx,
            send_stream,
        })
    }

    fn process_receiving_messages(
        mut byte_stream: Bytes<TcpStream>,
        server_address: SocketAddr,
        message_notifier_tx: Sender<Message>,
    ) -> Result<(), ExchangeError> {
        message_notifier_tx
            .send(Message::Connected)
            .map_err(|e| SendNotifyError(server_address, e.to_string()))?;

        while let Ok(bytes) = decode_bytes(&mut byte_stream) {
            message_notifier_tx
                .send(Message::Bytes(bytes))
                .map_err(|e| SendNotifyError(server_address, e.to_string()))?;
        }

        message_notifier_tx
            .send(Message::Disconnected)
            .map_err(|e| SendNotifyError(server_address, e.to_string()))
    }

    pub fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        TcpClient::send_by_stream(&mut self.send_stream, bytes)
    }

    pub fn send_by_stream(stream: &mut TcpStream, bytes: &[u8]) -> io::Result<()> {
        let encoded = &encode_bytes(bytes);
        stream.write_all(encoded)
    }

    pub fn clone_stream(&self) -> io::Result<TcpStream> {
        self.send_stream.try_clone()
    }
}
