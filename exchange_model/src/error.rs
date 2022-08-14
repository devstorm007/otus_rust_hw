use std::io;
use std::net::SocketAddr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExchangeError {
    #[error("Unexpected connect error")]
    ConnectError,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Sending notify message for '{0}' failed: {1}")]
    SendNotifyError(SocketAddr, String),
}
