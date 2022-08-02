use std::net::SocketAddr;
use std::sync::mpsc::Sender;

use crate::error::ProcessError;
use crate::error::ProcessError::SendNotifyError;

pub enum Message {
  Connected,
  Bytes(Vec<u8>),
  Disconnected,
}

pub struct NotifyMessage {
  pub message: Message,
  pub address: SocketAddr,
  pub message_sender_tx: Sender<Vec<u8>>,
}

impl NotifyMessage {
  pub fn new(message: Message, address: SocketAddr, tx: Sender<Vec<u8>>) -> NotifyMessage {
    NotifyMessage {
      message,
      address,
      message_sender_tx: tx,
    }
  }

  pub fn reply(&self, msg: Vec<u8>) -> Result<(), ProcessError> {
    self
      .message_sender_tx
      .send(msg)
      .map_err(|e| SendNotifyError(self.address, e.to_string()))
  }

  pub fn reply2(
    msg: Vec<u8>,
    message_sender_tx: Sender<Vec<u8>>,
    address: SocketAddr,
  ) -> Result<(), ProcessError> {
    message_sender_tx
      .send(msg)
      .map_err(|e| SendNotifyError(address, e.to_string()))
  }
}
