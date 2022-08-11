use std::any::Any;
use std::io;
use std::net::SocketAddr;

use flexbuffers::{DeserializationError, SerializationError};
use thiserror::Error;

use house::errors::intelligent_house_error::IntelligentHouseError;
use tcp_exchange::error::TcpExchangeError;

#[derive(Debug, Error)]
pub enum HouseExchangeError {
  #[error("Decode message error: {0}")]
  DecodeError(#[from] DeserializationError),
  #[error("Encode message error: {0}")]
  EncodeError(#[from] SerializationError),
  #[error("Tcp exchange error")]
  TcpExchangeError(#[from] TcpExchangeError),
  #[error("IO error: {0}")]
  Io(#[from] io::Error),
  #[error("Complete error")]
  CompleteError(Box<dyn Any>),
  #[error("Receive error")]
  ReceiveError,
  #[error("Sending notify message for '{0}' failed: {1}")]
  SendNotifyError(SocketAddr, String),
  #[error("house interaction error: {0}")]
  IntelligentHouseError(#[from] IntelligentHouseError),
}
