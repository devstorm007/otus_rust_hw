pub mod service;

use crate::domain::AppState;
use actix_web::{web::Data, HttpResponse};
use house::errors::intelligent_house_error::IntelligentHouseError;
use house::house::domain::*;

pub async fn get_rooms(
    state: Data<AppState>, /*,
                           room: web::Path<String>*/
) -> HttpResponse {
    let rooms: Result<Vec<Room>, IntelligentHouseError> = state.data.get_rooms().await;
    match rooms {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err),
    }
}
