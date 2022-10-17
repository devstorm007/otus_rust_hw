use mongodb::Client;

use crate::actions::service::DataService;
use crate::db::db_device_inventory::DbDeviceInventory;
use crate::db::db_intelligent_house::DbIntelligentHouse;

#[derive(Clone)]
pub struct AppState {
    pub data: DataService<DbDeviceInventory, DbIntelligentHouse>,
}

impl AppState {
    pub fn new(db_client: Client) -> Self {
        let inventory = DbDeviceInventory::new(db_client.database("inventory"));
        let house = DbIntelligentHouse::new("Plaza house", db_client.database("house"));
        AppState {
            data: DataService::create(inventory, house),
        }
    }
}
