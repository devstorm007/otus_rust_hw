use house::errors::intelligent_house_error::IntelligentHouseError;
use house::errors::intelligent_house_error::IntelligentHouseError::HouseError;
use house::house::domain::Room;
use house::house::intelligent_house::IntelligentHouse;
use house::inventory::device_inventory::DeviceInventory;

#[derive(Clone)]
pub struct DataService<T: DeviceInventory, H: IntelligentHouse> {
    inventory: T,
    house: H,
}

impl<T: DeviceInventory, H: IntelligentHouse> DataService<T, H> {
    pub fn create(inventory: T, house: H) -> Self {
        DataService { inventory, house }
    }

    pub async fn get_rooms(&self) -> Result<Vec<Room>, IntelligentHouseError> {
        self.house.get_rooms().await.map_err(HouseError)
    }
}
