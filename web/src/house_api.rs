use std::net::ToSocketAddrs;

use crate::actions::*;
use crate::domain::AppState;
use crate::error::HouseApiError;
use crate::error::HouseApiError::IOError;
use actix_web::{web, web::Data, App, HttpServer};
//use house::inventory::memory_device_inventory::MemoryDeviceInventory;
use mongodb::Client;
use tokio::task;
use tokio::task::JoinHandle;

pub struct HouseAPI {
    server_handle: JoinHandle<()>,
}

impl HouseAPI {
    pub async fn start<Addrs: ToSocketAddrs + Send + 'static>(
        address: Addrs,
    ) -> Result<Self, HouseApiError> {
        let server_handle: JoinHandle<()> = tokio::spawn(async move {
            Self::execute(address)
                .await
                .unwrap_or_else(|error| eprintln!("house web server: starting failed: {error:?}"));
        });

        Ok(HouseAPI { server_handle })
    }

    async fn execute<Addrs: ToSocketAddrs>(address: Addrs) -> Result<(), HouseApiError> /*-> Server*/
    {
        let db_client = Client::with_uri_str("mongodb://root:example@localhost:27017").await?;
        let db = db_client.database("otus");
        let server = HttpServer::new(move || {
            App::new()
                .app_data(Data::new(AppState::new(db.clone())))
                /*.service(web::resource("/users").route(web::post().to(users::web::save_new)))*/
                .service(web::resource("/rooms").route(web::get().to(get_rooms)))
        })
        .bind(address)
        .unwrap()
        .run();

        server.await.map_err(IOError)
    }

    pub async fn wait(self) -> Result<(), task::JoinError> {
        self.server_handle.await
    }
}
