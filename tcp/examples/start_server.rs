use std::error::Error;
use std::thread;

use threadpool::ThreadPool;

use tcp_exchange::domain::{Message, NotifyMessage};
use tcp_exchange::tcp_server::TcpServer;

fn main() -> Result<(), Box<dyn Error>> {
    let pool: ThreadPool = ThreadPool::default();

    let server_address = "127.0.0.1:45932";
    let server = TcpServer::start(server_address, &pool)?;
    let server_thread = thread::spawn(move || {
        while let Ok(notify) = server.messages.recv() {
            match notify.message {
                Message::Connected => println!("server: client {} connected", notify.address),
                Message::Bytes(ref bytes) => {
                    /*println!(
                        "server: received bytes from client {}: {bytes:?}",
                        msg.address
                    );*/
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
                        Err(error) => eprintln!(
                            "server: bad decoding from client '{}' failed: {error:?}",
                            notify.address
                        ),
                    }
                }
                Message::Disconnected => println!("server: client {} disconnected", notify.address),
            };
        }
    });

    server_thread.join().unwrap();

    Ok(())
}
