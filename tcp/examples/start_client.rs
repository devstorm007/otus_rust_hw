use std::error::Error;
use std::thread;
use tcp_exchange::domain::Message;
use tcp_exchange::tcp_client::TcpClient;
use threadpool::ThreadPool;

fn main() -> Result<(), Box<dyn Error>> {
    let pool: ThreadPool = ThreadPool::default();

    let server_address = "127.0.0.1:45932";

    let mut client = TcpClient::connect(server_address, &pool)?;
    let client_thread = thread::spawn(move || {
        while let Ok(msg) = client.messages.recv() {
            println!("client2: message received for client2 {}", client.address);
            match msg {
                Message::Connected => {
                    println!("client2: connected to server '{server_address}'");
                    client
                        .send("hello server".as_bytes())
                        .unwrap_or_else(|error| {
                            eprintln!(
                                "client2: send message to server '{server_address}' failed: {error:?}"
                            )
                        });
                }
                Message::Bytes(bytes) => {
                    println!("client2: received bytes from server {server_address}: {bytes:?}");
                    match String::from_utf8(bytes) {
                        Ok(_data) => {
                            println!("client: receive '{_data:?}' from server {}", server_address)
                        }
                        Err(error) => eprintln!(
                        "client2: bad decoding from server '{server_address}' failed: {error:?}"
                    ),
                    }
                }
                Message::Disconnected => println!("client: disconnected from '{server_address}'"),
            };
        }
    });

    client_thread.join().unwrap();

    Ok(())
}
