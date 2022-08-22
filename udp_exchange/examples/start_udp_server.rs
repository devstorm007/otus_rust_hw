use std::error::Error;

use threadpool::ThreadPool;

use exchange_protocol::domain::Message;
use udp_exchange::udp_server::UdpServer;

fn main() -> Result<(), Box<dyn Error>> {
    let pool: ThreadPool = ThreadPool::default();

    let server_address = "127.0.0.1:45959";
    let server = UdpServer::start(server_address, &pool)?;
    pool.execute(move || {
        while let Ok(notify) = server.messages.recv() {
            if let Message::Bytes(ref bytes) = notify.message {
                match String::from_utf8(bytes.clone()) {
                    Ok(_data) => {
                        println!(
                            "server: received '{_data:?}' from client {}",
                            notify.address
                        );
                        let answer = "hello client".as_bytes().to_vec();
                        notify.reply(answer).unwrap_or_else(|error| {
                            eprintln!(
                                "server: send message to client '{}' failed: {error:?}",
                                notify.address
                            );
                        })
                    }
                    Err(_error) => eprintln!(
                        "server: bad decoding from client '{}' failed: {_error:?}",
                        notify.address
                    ),
                }
            }
        }
    });

    pool.join();

    Ok(())
}
