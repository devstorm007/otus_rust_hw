use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use crate::error::ExchangeError;
use crate::error::ExchangeError::SendNotifyError;

pub enum Message {
    Connected,
    Bytes(Vec<u8>),
    Disconnected,
}

pub struct NotifyMessage {
    pub message: Message,
    pub address: SocketAddr,
    message_sender_tx: Sender<Vec<u8>>,
}

impl NotifyMessage {
    pub fn new(message: Message, address: SocketAddr, tx: Sender<Vec<u8>>) -> NotifyMessage {
        NotifyMessage {
            message,
            address,
            message_sender_tx: tx,
        }
    }

    pub fn reply(&self, msg: Vec<u8>) -> Result<(), ExchangeError> {
        self.message_sender_tx
            .send(msg)
            .map_err(|e| SendNotifyError(self.address, e.to_string()))
    }
}
