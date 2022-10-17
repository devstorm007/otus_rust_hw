use async_trait::async_trait;
use mongodb::Database;

use house::errors::intelligent_house_error::InventoryError;
use house::house::domain::{DeviceName, RoomName};
use house::inventory::device_inventory::DeviceInventory;
use house::inventory::domain::DeviceItem;

#[derive(Clone)]
pub struct DbDeviceInventory {
    db: Database, //room_devices: Arc<RwLock<HashMap<RoomName, HashMap<DeviceName, DeviceItem>>>>,
}

impl DbDeviceInventory {
    pub fn new(
        db: Database, //room_devices: HashMap<RoomName, HashMap<DeviceName, DeviceItem>>,
    ) -> DbDeviceInventory {
        DbDeviceInventory {
            db, //room_devices: Arc::new(RwLock::new(room_devices)),
        }
    }
}

#[async_trait]
impl DeviceInventory for DbDeviceInventory {
    async fn get_info(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<String, InventoryError> {
        todo!()
    }

    async fn get_rooms(&self) -> std::result::Result<Vec<RoomName>, InventoryError> {
        todo!()
    }

    async fn add_room(&self, room_name: &RoomName) -> Result<(), InventoryError> {
        todo!()
    }

    async fn remove_room(&self, room_name: &RoomName) -> Result<(), InventoryError> {
        todo!()
    }

    async fn add_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
        device: DeviceItem,
    ) -> Result<(), InventoryError> {
        todo!()
    }

    async fn remove_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), InventoryError> {
        todo!()
    }

    async fn change_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
        modify: impl Fn(DeviceItem) -> Result<DeviceItem, InventoryError> + Send,
    ) -> std::result::Result<(), InventoryError> {
        todo!()
    }

    async fn get_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<DeviceItem, InventoryError> {
        todo!()
    }
}
