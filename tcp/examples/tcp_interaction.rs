use std::error::Error;
use std::thread;
use tcp_exchange::domain::{Message, NotifyMessage};
use tcp_exchange::tcp_client::TcpClient;
use tcp_exchange::tcp_server::TcpServer;
use threadpool::ThreadPool;

fn main() -> Result<(), Box<dyn Error>> {
    let pool: ThreadPool = ThreadPool::default();

    let server_address = "127.0.0.1:45932";
    let server = TcpServer::start(server_address, &pool)?;
    let server_thread = thread::spawn(move || {
        while let Ok(msg) = server.messages.recv() {
            match msg.message {
                Message::Connected => println!("server: client {} connected", msg.address),
                Message::Bytes(bytes) => {
                    println!(
                        "server: received bytes from client {}: {bytes:?}",
                        msg.address
                    );
                    match String::from_utf8(bytes) {
                        Ok(_data) => {
                            println!("server: received '{_data:?}' from client {}", msg.address);
                            let answer = "hello client".as_bytes().to_vec();
                            NotifyMessage::reply2(answer, msg.message_sender_tx, msg.address)
                                .unwrap_or_else(|error| {
                                    eprintln!(
                                        "server: send message to client '{}' failed: {error:?}",
                                        msg.address
                                    )
                                });
                        }
                        Err(error) => eprintln!(
                            "server: bad decoding from client '{}' failed: {error:?}",
                            msg.address
                        ),
                    }
                }
                Message::Disconnected => println!("server: client {} disconnected", msg.address),
            };
        }
    });

    let mut client = TcpClient::connect(server_address, &pool)?;
    let client_thread = thread::spawn(move || {
        while let Ok(msg) = client.messages.recv() {
            println!("message received for client {}", client.address);
            match msg {
                Message::Connected => {
                    println!("client: connected to server '{server_address}'");
                    client
                        .send("hello server".as_bytes())
                        .unwrap_or_else(|error| {
                            eprintln!(
                                "client: send message to server '{server_address}' failed: {error:?}"
                            )
                        });
                }
                Message::Bytes(bytes) => {
                    println!("client: received bytes from server {server_address}: {bytes:?}");
                    match String::from_utf8(bytes) {
                        Ok(_data) => {
                            println!("client: receive '{_data:?}' from server {}", server_address)
                        }
                        Err(error) => eprintln!(
                            "client: bad decoding from server '{server_address}' failed: {error:?}"
                        ),
                    }
                }
                Message::Disconnected => println!("client: disconnected from '{server_address}'"),
            };
        }
    });

    client_thread.join().unwrap();
    server_thread.join().unwrap();

    Ok(())
}
