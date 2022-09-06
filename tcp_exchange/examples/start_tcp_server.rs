use std::error::Error;

use exchange_protocol::domain::Message;
use tcp_exchange::tcp_server::TcpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = TcpServer::start("127.0.0.1:45932").await?;

    let receiver = server.messages.clone();
    while let Some(notify) = receiver.lock().await.recv().await {
        match notify.message {
            Message::Connected => println!("server: client {} connected", notify.address),
            Message::Bytes(ref bytes) => match String::from_utf8(bytes.clone()) {
                Ok(_data) => {
                    println!(
                        "tcp_server: received {_data:?} from client {}",
                        notify.address
                    );
                    let answer = "hello client".as_bytes().to_vec();
                    notify.reply(answer).await.unwrap_or_else(|error| {
                        eprintln!(
                            "tcp_server: send message to client '{}' failed: {error:?}",
                            notify.address
                        );
                    })
                }
                Err(_error) => eprintln!(
                    "server: bad decoding from client '{}' failed: {_error:?}",
                    notify.address
                ),
            },
            Message::Disconnected => println!("server: client {} disconnected", notify.address),
        };
    }

    Ok(())
}
