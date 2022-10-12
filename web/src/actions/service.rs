use house::errors::intelligent_house_error::InventoryError;
use house::house::domain::RoomName;
use house::house::house_storage::HouseStorage;
use house::inventory::device_inventory::DeviceInventory;

#[derive(Clone)]
pub struct DataService<T: DeviceInventory, H: HouseStorage> {
    inventory: T,
    house: H,
}

impl<T: DeviceInventory, H: HouseStorage> DataService<T, H> {
    pub fn create(inventory: T, house: H) -> Self {
        DataService { inventory, house }
    }

    pub async fn get_rooms(&self) -> Result<Vec<RoomName>, InventoryError> {
        let rooms = self.house.get_rooms();
        Ok(rooms)
    }
}
