use house::devices::power_socket::PowerSocket;
use house::devices::temperature_sensor::TemperatureSensor;
use house::errors::intelligent_house_error::HouseError::RoomAlreadyAdded;
use house::errors::intelligent_house_error::IntelligentHouseError;
use house::errors::intelligent_house_error::IntelligentHouseError::{HouseErr, InventoryErr};
use house::house::domain::{DeviceName, Room, RoomName};
use house::house::intelligent_house::IntelligentHouse;
use house::inventory::device_inventory::DeviceInventory;
use house::inventory::domain::{DeviceItem, RoomDevices};

#[derive(Clone)]
pub struct DataService<T: DeviceInventory + Sync, H: IntelligentHouse> {
    inventory: T,
    house: H,
}

impl<T: DeviceInventory + Sync, H: IntelligentHouse> DataService<T, H> {
    pub fn create(inventory: T, house: H) -> Self {
        DataService { inventory, house }
    }

    pub async fn get_rooms(&self) -> Result<Vec<Room>, IntelligentHouseError> {
        self.house.get_rooms().await.map_err(HouseErr)
    }

    pub async fn add_room(&self, room_name: RoomName) -> Result<(), IntelligentHouseError> {
        if self.house.get_room(&room_name).await.is_ok() {
            return Err(HouseErr(RoomAlreadyAdded(room_name)));
        }
        self.house.add_room(&room_name).await?;
        self.inventory
            .add_room(&room_name)
            .await
            .map_err(InventoryErr)
    }

    pub async fn delete_room(&self, room_name: RoomName) -> Result<(), IntelligentHouseError> {
        self.house.remove_room(&room_name).await?;
        self.inventory
            .remove_room(&room_name)
            .await
            .map_err(InventoryErr)
    }

    pub async fn get_room_devices(
        &self,
        room_name: RoomName,
    ) -> Result<Vec<DeviceName>, IntelligentHouseError> {
        self.house.get_devices(&room_name).await.map_err(HouseErr)
    }

    pub async fn add_room_device(
        &self,
        room_name: RoomName,
        device_name: DeviceName,
    ) -> Result<(), IntelligentHouseError> {
        self.house
            .add_device(&room_name, &device_name)
            .await
            .map_err(HouseErr)
    }

    pub async fn delete_room_device(
        &self,
        room_name: RoomName,
        device_name: DeviceName,
    ) -> Result<(), IntelligentHouseError> {
        self.house
            .remove_device(&room_name, &device_name)
            .await
            .map_err(HouseErr)
    }

    pub async fn get_inventory_devices(&self) -> Result<Vec<RoomDevices>, IntelligentHouseError> {
        self.inventory
            .get_all_room_devices()
            .await
            .map_err(InventoryErr)
    }

    pub async fn add_inventory_socket(
        &self,
        room_name: RoomName,
        device_name: DeviceName,
        socket: PowerSocket,
    ) -> Result<(), IntelligentHouseError> {
        self.inventory
            .add_device(&room_name, &device_name, DeviceItem::inject(socket))
            .await
            .map_err(InventoryErr)
    }

    pub async fn add_inventory_sensor(
        &self,
        room_name: RoomName,
        device_name: DeviceName,
        sensor: TemperatureSensor,
    ) -> Result<(), IntelligentHouseError> {
        self.inventory
            .add_device(&room_name, &device_name, DeviceItem::inject(sensor))
            .await
            .map_err(InventoryErr)
    }

    pub async fn delete_inventory_device(
        &self,
        room_name: RoomName,
        device_name: DeviceName,
    ) -> Result<(), IntelligentHouseError> {
        self.inventory
            .remove_device(&room_name, &device_name)
            .await
            .map_err(InventoryErr)
    }

    pub async fn get_house_report(&self) -> Result<String, IntelligentHouseError> {
        //let result = self.house.generate_report(&self.inventory).await?;
        //Ok(result.split("\n").map(|s| s.to_string()).collect())
        self.house
            .generate_report(&self.inventory)
            .await
            .map_err(HouseErr)
    }
}
