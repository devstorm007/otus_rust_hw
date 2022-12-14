use std::io;
use std::net::SocketAddr;

use flexbuffers::{DeserializationError, SerializationError};
use thiserror::Error;

use exchange_protocol::error::ExchangeError;
use house::errors::intelligent_house_error::IntelligentHouseError;

#[derive(Debug, Error)]
pub enum HouseExchangeError {
    #[error("Decode message error: {0}")]
    DecodeError(#[from] DeserializationError),
    #[error("Encode message error: {0}")]
    EncodeError(#[from] SerializationError),
    #[error("Tcp exchange error")]
    TcpExchangeError(#[from] ExchangeError),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Receive error")]
    ReceiveError,
    #[error("Sending notify message for '{0}' failed: {1}")]
    SendNotifyError(SocketAddr, String),
    #[error("Sending notify message to bus failed: {0}")]
    SendNotifyEventError(String),
    #[error("house interaction error: {0}")]
    IntelligentHouseError(#[from] IntelligentHouseError),
}
