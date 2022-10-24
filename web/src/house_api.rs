use std::net::ToSocketAddrs;

use actix_web::{web, web::Data, App, HttpResponse, HttpServer};
use mongodb::Client;
use tokio::task;
use tokio::task::JoinHandle;

use crate::actions::*;
use crate::domain::AppState;
use crate::error::HouseApiError;
use crate::error::HouseApiError::IOError;

pub struct HouseAPI {
    server_handle: JoinHandle<()>,
}

impl HouseAPI {
    pub async fn start<Addrs: ToSocketAddrs + Send + 'static>(
        address: Addrs,
        connection: String,
        drop_db: bool,
    ) -> Result<Self, HouseApiError> {
        let server_handle: JoinHandle<()> = tokio::spawn(async move {
            Self::execute(address, connection, drop_db)
                .await
                .unwrap_or_else(|error| eprintln!("house web server: starting failed: {error:?}"));
        });

        Ok(HouseAPI { server_handle })
    }

    async fn execute<Addrs: ToSocketAddrs>(
        address: Addrs,
        db_connection: String,
        drop_db: bool,
    ) -> Result<(), HouseApiError> {
        let db_client = Client::with_uri_str(db_connection).await?;
        if drop_db {
            db_client.database("inventory").drop(None).await.unwrap();
            db_client.database("house").drop(None).await.unwrap();
        }

        let server = HttpServer::new(move || {
            App::new()
                .app_data(Data::new(AppState::new(db_client.clone())))
                .service(web::resource("/readiness").route(web::get().to(HttpResponse::Ok)))
                .service(
                    web::scope("/rooms")
                        .service(web::resource("").route(web::get().to(get_rooms)))
                        .service(
                            web::scope("/{name}")
                                .service(
                                    web::resource("")
                                        .route(web::post().to(add_room))
                                        .route(web::delete().to(delete_room)),
                                )
                                .service(
                                    web::scope("/devices")
                                        .service(
                                            web::resource("")
                                                .route(web::get().to(get_room_devices)),
                                        )
                                        .service(
                                            web::resource("/{device_name}")
                                                .route(web::post().to(add_room_device))
                                                .route(web::delete().to(delete_room_device)),
                                        ),
                                ),
                        ),
                )
                .service(
                    web::scope("/inventory")
                        .service(web::resource("").route(web::get().to(get_inventory_devices)))
                        .service(
                            web::scope("/{room_name}/devices")
                                .service(
                                    web::resource("/socket/{device_name}")
                                        .route(web::post().to(add_socket)),
                                )
                                .service(
                                    web::resource("/sensor/{device_name}")
                                        .route(web::post().to(add_sensor)),
                                )
                                .service(
                                    web::resource("/{device_name}")
                                        .route(web::delete().to(delete_inventory_device)),
                                ),
                        ),
                )
                .service(web::resource("/report").route(web::get().to(get_house_report)))
        })
        .bind(address)?
        .run();

        server.await.map_err(IOError)
    }

    pub async fn wait(self) -> Result<(), task::JoinError> {
        self.server_handle.await
    }
}
