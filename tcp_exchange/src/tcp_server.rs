use std::io::Bytes;
use std::io::{Read, Write};
use std::net::{
  Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream, ToSocketAddrs,
};
use std::sync::mpsc::{channel, Receiver, Sender};

use threadpool::ThreadPool;

use crate::domain::*;
use crate::error::TcpExchangeError;
use crate::error::TcpExchangeError::SendNotifyError;
use crate::tcp_protocol::{decode_bytes, encode_bytes};

pub struct TcpServer {
  pub address: SocketAddr,
  pub messages: Receiver<NotifyMessage>,
}

impl TcpServer {
  pub fn start<Addrs: ToSocketAddrs>(
    address: Addrs,
    pool: &ThreadPool,
  ) -> Result<TcpServer, TcpExchangeError> {
    let listener = TcpListener::bind(address)?;
    let server_address = listener.local_addr()?;

    let (message_notifier_tx, message_notifier_rx) = channel();

    let pool_clone = pool.clone();
    pool.execute(move || {
      listener
        .incoming()
        .filter_map(Result::ok)
        .for_each(|stream| {
          let client_address = stream.peer_addr().unwrap_or_else(|e| {
            eprintln!("getting client address failed: {e}");
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0))
          });

          TcpServer::start_stream_processing(
            stream,
            client_address,
            message_notifier_tx.clone(),
            &pool_clone,
          )
          .unwrap_or_else(|error| {
            eprintln!("start stream processing failed for '{client_address}': {error:?}")
          });
        });
    });

    println!("tcp server started at {}", server_address.clone());

    Ok(TcpServer {
      address: server_address,
      messages: message_notifier_rx,
    })
  }

  fn start_stream_processing(
    client_stream: TcpStream,
    client_address: SocketAddr,
    message_notifier_tx: Sender<NotifyMessage>,
    pool: &ThreadPool,
  ) -> Result<(), TcpExchangeError> {
    let (client_sender_tx, client_sender_rx) = channel::<Vec<u8>>();

    let send_client_stream = client_stream.try_clone()?;
    pool.execute(move || {
      TcpServer::process_sending_messages(send_client_stream, client_address, client_sender_rx);
    });

    message_notifier_tx
      .send(NotifyMessage::new(
        Message::Connected,
        client_address,
        client_sender_tx.clone(),
      ))
      .map_err(|e| SendNotifyError(client_address, e.to_string()))?;

    let receive_client_stream = client_stream.try_clone()?;
    pool.execute(move || {
      TcpServer::process_receiving_messages(
        receive_client_stream.bytes(),
        client_address,
        client_sender_tx.clone(),
        message_notifier_tx.clone(),
      )
      .unwrap_or_else(|error| {
        eprintln!("receiving messages failed for '{client_address}': {error:?}")
      });

      client_stream.shutdown(Shutdown::Both).unwrap_or_default();
    });

    Ok(())
  }

  fn process_sending_messages(
    mut send_client_stream: TcpStream,
    _client_address: SocketAddr,
    client_sender_rx: Receiver<Vec<u8>>,
  ) {
    while let Ok(msg_bytes) = client_sender_rx.recv() {
      let encoded = encode_bytes(msg_bytes.as_slice());
      send_client_stream
        .write_all(&encoded)
        .unwrap_or_else(|error| {
          eprintln!("sending message to client '{_client_address}' failed: {error}")
        });
    }
  }

  fn process_receiving_messages(
    mut client_bytes: Bytes<TcpStream>,
    client_address: SocketAddr,
    client_sender_tx: Sender<Vec<u8>>,
    message_notifier_tx: Sender<NotifyMessage>,
  ) -> Result<(), TcpExchangeError> {
    while let Ok(message) = decode_bytes(&mut client_bytes) {
      message_notifier_tx
        .send(NotifyMessage::new(
          Message::Bytes(message),
          client_address,
          client_sender_tx.clone(),
        ))
        .map_err(|e| SendNotifyError(client_address, e.to_string()))?;
    }

    message_notifier_tx
      .send(NotifyMessage::new(
        Message::Disconnected,
        client_address,
        client_sender_tx,
      ))
      .map_err(|e| SendNotifyError(client_address, e.to_string()))
  }
}
