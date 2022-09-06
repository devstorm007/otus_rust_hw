use std::error::Error;

use exchange_protocol::domain::Message;
use udp_exchange::udp_server::UdpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server_address = "127.0.0.1:45959";
    let mut server = UdpServer::start(server_address).await?;

    while let Some(notify) = server.messages.recv().await {
        if let Message::Bytes(ref bytes) = notify.message {
            match String::from_utf8(bytes.clone()) {
                Ok(_data) => {
                    let answer = "hello client".as_bytes().to_vec();
                    notify.reply(answer).await.unwrap_or_else(|error| {
                        eprintln!(
                            "udp_server: send message to client '{}' failed: {error:?}",
                            notify.address
                        );
                    })
                }
                Err(_error) => eprintln!(
                    "udp_server: bad decoding from client '{}' failed: {_error:?}",
                    notify.address
                ),
            }
        }
    }

    Ok(())
}
