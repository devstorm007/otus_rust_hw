use std::io;
use std::io::Bytes;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};

use threadpool::ThreadPool;

use crate::domain::*;
use crate::error::ProcessError;
use crate::error::ProcessError::SendNotifyError;
use crate::tcp_protocol::{decode_bytes, encode_bytes};

pub struct TcpClient {
    pub address: SocketAddr,
    pub messages: Receiver<Message>,
    send_stream: TcpStream,
}

impl TcpClient {
    pub fn connect<T: ToSocketAddrs>(
        address: T,
        pool: &ThreadPool,
    ) -> Result<TcpClient, ProcessError> {
        let stream = TcpStream::connect(address)?;
        let server_address = stream.peer_addr()?;
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

            stream
                .shutdown(Shutdown::Both)
                .unwrap_or_else(|error| eprintln!("client shutdown failed: {error:?}"));
        });

        Ok(TcpClient {
            address: client_address,
            messages: message_notifier_rx,
            send_stream,
        })
    }

    fn process_receiving_messages(
        mut bytes: Bytes<TcpStream>,
        server_address: SocketAddr,
        message_notifier_tx: Sender<Message>,
    ) -> Result<(), ProcessError> {
        message_notifier_tx
            .send(Message::Connected)
            .map_err(|e| SendNotifyError(server_address, e.to_string()))?;

        while let Ok(message) = decode_bytes(&mut bytes) {
            message_notifier_tx
                .send(Message::Bytes(message))
                .map_err(|e| SendNotifyError(server_address, e.to_string()))?;
        }

        message_notifier_tx
            .send(Message::Disconnected)
            .map_err(|e| SendNotifyError(server_address, e.to_string()))
    }

    pub fn send(&mut self, msg: &[u8]) -> Result<(), io::Error> {
        self.send_stream.write_all(&encode_bytes(msg))
    }
}
