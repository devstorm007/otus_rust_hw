use std::io;
use std::net::SocketAddr;

use exchange_protocol::error::ExchangeError;
use house::errors::intelligent_house_error::IntelligentHouseError;
use thiserror::Error;
use tokio::task;

#[derive(Debug, Error)]
pub enum HouseApiError {
    #[error("Tcp exchange error")]
    TcpExchangeError(#[from] ExchangeError),
    #[error("Db error: {0}")]
    DBError(#[from] mongodb::error::Error),
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
    #[error("Receive error")]
    ReceiveError,
    #[error("Sending notify message for '{0}' failed: {1}")]
    SendNotifyError(SocketAddr, String),
    #[error("Sending notify message to bus failed: {0}")]
    SendNotifyEventError(String),
    #[error("house interaction error: {0}")]
    IntelligentHouseError(#[from] IntelligentHouseError),
    #[error("api executing error: {0}")]
    JoinError(#[from] task::JoinError),
}
