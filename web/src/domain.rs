use house::inventory::device_inventory::DeviceInventory;
use mongodb::Database;

#[derive(Clone)]
pub struct AppState<T: DeviceInventory> {
    pub database: Database,
    pub device_inventory: T,
}
