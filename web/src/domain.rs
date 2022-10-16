use crate::actions::service::DataService;
use house::house::intelligent_house::IntelligentHouse;
use house::inventory::memory_device_inventory::MemoryDeviceInventory;
use mongodb::Database;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    pub data: DataService<MemoryDeviceInventory, IntelligentHouse>,
}

impl AppState {
    pub fn new(_db: Database) -> Self {
        let inventory = MemoryDeviceInventory::new(HashMap::new());
        let house = IntelligentHouse::create("", vec![]);
        AppState {
            data: DataService::create(inventory, house),
        }
    }
}
