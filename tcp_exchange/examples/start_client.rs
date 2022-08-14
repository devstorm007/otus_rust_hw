use std::error::Error;
use std::thread;

use threadpool::ThreadPool;

use exchange_model::domain::Message;
use tcp_exchange::tcp_client::TcpClient;

fn main() -> Result<(), Box<dyn Error>> {
    let pool: ThreadPool = ThreadPool::default();

    let server_address = "127.0.0.1:45932";

    let mut client = TcpClient::connect(server_address, &pool)?;
    let client_thread = thread::spawn(move || {
        while let Ok(msg) = client.messages.recv() {
            match msg {
                Message::Connected => {
                    println!("client: connected to server '{server_address}'");
                    client
                        .send("hello server".as_bytes())
                        .unwrap_or_else(|error| {
                            eprintln!("client: send message to server '{server_address}' failed: {error:?}")
                        });
                }
                Message::Bytes(bytes) => match String::from_utf8(bytes) {
                    Ok(_data) => {
                        println!(
                            "client: received '{_data:?}' from server {}",
                            server_address
                        )
                    }
                    Err(_error) => {
                        eprintln!("client: bad decoding from server '{server_address}' failed: {_error:?}")
                    }
                },
                Message::Disconnected => println!("client: disconnected from '{server_address}'"),
            };
        }
    });

    client_thread.join().unwrap();

    Ok(())
}
