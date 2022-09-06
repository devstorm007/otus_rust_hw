use std::error::Error;

use exchange_protocol::domain::Message;
use tcp_exchange::tcp_client::TcpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server_address = "127.0.0.1:45932";
    let mut client = TcpClient::connect(server_address).await?;

    let receiver = client.messages.clone();
    while let Some(msg) = receiver.lock().await.recv().await {
        match msg {
            Message::Connected => {
                println!("client: connected to server '{server_address}'");
                client
                    .send("hello server".as_bytes())
                    .await
                    .unwrap_or_else(|error| {
                        eprintln!(
                            "client: send message to server '{server_address}' failed: {error:?}"
                        )
                    });
            }
            Message::Bytes(bytes) => {
                match String::from_utf8(bytes) {
                    Ok(_data) => {
                        println!(
                            "client: received '{_data:?}' from server {}",
                            server_address
                        )
                    }
                    Err(_error) => {
                        eprintln!("client: bad decoding from server '{server_address}' failed: {_error:?}")
                    }
                }
            }
            Message::Disconnected => println!("client: disconnected from '{server_address}'"),
        };
    }

    Ok(())
}
