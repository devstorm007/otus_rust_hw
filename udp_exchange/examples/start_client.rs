use std::error::Error;
//use std::thread;

use threadpool::ThreadPool;

//use exchange_protocol::domain::Message;

fn main() -> Result<(), Box<dyn Error>> {
    let pool: ThreadPool = ThreadPool::default();

    //let server_address = "127.0.0.1:45932";

    pool.join();

    Ok(())
}