use std::error::Error;
//use std::thread;

//use exchange_protocol::domain::Message;
use threadpool::ThreadPool;

fn main() -> Result<(), Box<dyn Error>> {
    let pool: ThreadPool = ThreadPool::default();

    //let server_address = "127.0.0.1:45999";

    pool.join();

    Ok(())
}
