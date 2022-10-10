use crate::domain::AppState;
use actix_web::{web::Data, HttpResponse};
use house::errors::intelligent_house_error::InventoryError;
use house::house::intelligent_house::RoomName;
use house::inventory::device_inventory::DeviceInventory;

pub async fn get_all<T: DeviceInventory>(
    state: Data<AppState<T>>, /*,
                              room: web::Path<String>*/
) -> HttpResponse {
    let rooms: Result<Vec<RoomName>, InventoryError> = state.device_inventory.get_rooms().await;
    match rooms {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
