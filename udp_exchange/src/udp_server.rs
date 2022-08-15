use std::io::Bytes;
use std::io::{Read, Write};
use std::net::{
    Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream, ToSocketAddrs, UdpSocket,
};
use std::sync::mpsc::{channel, Receiver, Sender};

use threadpool::ThreadPool;

use exchange_protocol::domain::{Message, NotifyMessage};
use exchange_protocol::error::ExchangeError;
use exchange_protocol::error::ExchangeError::SendNotifyError;

use exchange_protocol::codecs::{decode_bytes, encode_bytes};

pub struct UdpServer {
    pub address: SocketAddr,
    pub messages: Receiver<NotifyMessage>,
}

impl UdpServer {
    pub fn start<Addrs: ToSocketAddrs>(
        address: Addrs,
        pool: &ThreadPool,
    ) -> Result<UdpServer, ExchangeError> {
        let socket = UdpSocket::bind(address)?;
        let server_address = socket.local_addr()?;

        let (message_notifier_tx, message_notifier_rx) = channel::<NotifyMessage>();

        let pool_clone = pool.clone();
        pool.execute(move || {
            let client_address = socket.peer_addr().unwrap_or_else(|e| {
                eprintln!("getting client address failed: {e}");
                SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0))
            });
            UdpServer::process_stream(
                socket,
                //stream,
                client_address,
                message_notifier_tx.clone(),
                &pool_clone,
            )
            .unwrap_or_else(|error| {
                eprintln!("process stream failed for '{client_address}': {error:?}")
            });
        });

        println!("udp server started at {}", server_address.clone());

        Ok(UdpServer {
            address: server_address,
            messages: message_notifier_rx,
        })
    }

    fn process_sending_messages(
        socket: UdpSocket,
        _client_address: SocketAddr,
        client_sender_rx: Receiver<Vec<u8>>,
    ) {
        let buf = &mut buf[..10];
        buf.reverse();
        socket.send_to(buf, &src)?;

        while let Ok(msg_bytes) = client_sender_rx.recv() {
            let encoded = encode_bytes(msg_bytes.as_slice());
            send_client_stream
                .write_all(&encoded)
                .unwrap_or_else(|error| {
                    eprintln!("sending message to client '{_client_address}' failed: {error}")
                });
        }
    }

    fn process_stream(
        //client_stream: TcpStream,
        socket: UdpSocket,
        client_address: SocketAddr,
        message_notifier_tx: Sender<NotifyMessage>,
        pool: &ThreadPool,
    ) -> Result<(), ExchangeError> {
        let mut buf = [0; 10];
        let (amt, src) = socket.recv_from(&mut buf)?;

        todo!()
    }
}
