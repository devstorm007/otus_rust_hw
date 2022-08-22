use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use crate::error::ExchangeError;
use crate::error::ExchangeError::SendNotifyError;

#[derive(Debug)]
pub enum Message {
    Connected,
    Bytes(Vec<u8>),
    Disconnected,
}

pub struct SendMessage {
    pub bytes: Vec<u8>,
    pub client_address: SocketAddr,
}

pub struct NotifyMessage {
    pub message: Message,
    pub address: SocketAddr,
    message_sender_tx: Sender<SendMessage>,
}

impl NotifyMessage {
    pub fn new(message: Message, address: SocketAddr, tx: Sender<SendMessage>) -> NotifyMessage {
        NotifyMessage {
            message,
            address,
            message_sender_tx: tx,
        }
    }

    pub fn reply(&self, bytes: Vec<u8>) -> Result<(), ExchangeError> {
        self.message_sender_tx
            .send(SendMessage {
                bytes,
                client_address: self.address,
            })
            .map_err(|e| SendNotifyError(self.address, e.to_string()))
    }
}
