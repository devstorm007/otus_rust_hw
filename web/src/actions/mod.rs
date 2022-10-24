pub mod service;

use crate::domain::AppState;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use house::devices::power_socket::PowerSocket;
use house::devices::temperature_sensor::TemperatureSensor;
use house::house::domain::*;

pub async fn get_rooms(state: Data<AppState>) -> HttpResponse {
    match state.data.get_rooms().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn add_room(state: Data<AppState>, name: Path<RoomName>) -> HttpResponse {
    match state.data.add_room(name.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn delete_room(state: Data<AppState>, name: Path<RoomName>) -> HttpResponse {
    match state.data.delete_room(name.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn get_room_devices(state: Data<AppState>, name: Path<RoomName>) -> HttpResponse {
    match state.data.get_room_devices(name.into_inner()).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn add_room_device(
    state: Data<AppState>,
    params: Path<(RoomName, DeviceName)>,
) -> HttpResponse {
    let (room_name, device_name) = params.into_inner();
    match state.data.add_room_device(room_name, device_name).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn delete_room_device(
    state: Data<AppState>,
    params: Path<(RoomName, DeviceName)>,
) -> HttpResponse {
    let (room_name, device_name) = params.into_inner();
    match state.data.delete_room_device(room_name, device_name).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn get_house_report(state: Data<AppState>) -> HttpResponse {
    match state.data.get_house_report().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn get_inventory_devices(state: Data<AppState>) -> HttpResponse {
    match state.data.get_inventory_devices().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn add_socket(
    state: Data<AppState>,
    params: Path<(RoomName, DeviceName)>,
    socket: Json<PowerSocket>,
) -> HttpResponse {
    let (room_name, device_name) = params.into_inner();
    match state
        .data
        .add_inventory_socket(room_name, device_name, socket.into_inner())
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn add_sensor(
    state: Data<AppState>,
    params: Path<(RoomName, DeviceName)>,
    sensor: Json<TemperatureSensor>,
) -> HttpResponse {
    let (room_name, device_name) = params.into_inner();
    match state
        .data
        .add_inventory_sensor(room_name, device_name, sensor.into_inner())
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}

pub async fn delete_inventory_device(
    state: Data<AppState>,
    params: Path<(RoomName, DeviceName)>,
) -> HttpResponse {
    let (room_name, device_name) = params.into_inner();
    match state
        .data
        .delete_inventory_device(room_name, device_name)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}
