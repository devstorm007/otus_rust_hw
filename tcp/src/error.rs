use std::io;
use std::net::SocketAddr;
use thiserror::Error;

pub type ConnectResult<T> = Result<T, ConnectError>;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Unexpected connect error")]
    ConnectError,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Sending notify message for '{0}' failed: {1}")]
    SendNotifyError(SocketAddr, String),
}

/// Connection error. Includes IO and handshake error.
#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Unexpected handshake response: {0}")]
    BadHandshake(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub type SendResult = Result<(), SendError>;

/// Send data error
#[derive(Debug, Error)]
pub enum SendError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub type RecvResult = Result<String, RecvError>;

/// Send data error. Includes IO and encoding error.
#[derive(Debug, Error)]
pub enum RecvError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("bad encoding")]
    BadEncoding,
}
