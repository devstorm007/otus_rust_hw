use std::error::Error;

use threadpool::ThreadPool;

use exchange_protocol::domain::Message;
use upd_exchange::udp_client::UdpClient;

fn main() -> Result<(), Box<dyn Error>> {
    let pool: ThreadPool = ThreadPool::default();

    let local_address = "127.0.0.1:45858";
    let server_address = "127.0.0.1:45959";
    let mut client = UdpClient::connect(server_address, local_address, &pool)?;

    pool.execute(move || {
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

    pool.join();

    Ok(())
}
