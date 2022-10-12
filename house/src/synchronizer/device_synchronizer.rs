use anyhow::Result;
use std::sync::Arc;

use crate::errors::intelligent_house_error::IntelligentHouseError;
use crate::house::domain::*;
use crate::house::house_storage::*;
use crate::house::intelligent_house::*;
use crate::inventory::device_inventory::DeviceInventory;
use crate::DeviceItem;
use async_trait::async_trait;
use tokio::sync::Mutex;

#[async_trait]
pub trait DeviceSynchronizer {
    async fn add_room(&mut self, room_name: &RoomName) -> Result<(), IntelligentHouseError>;

    async fn remove_room(&mut self, room_name: &RoomName) -> Result<(), IntelligentHouseError>;

    async fn add_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
        device: DeviceItem,
    ) -> Result<(), IntelligentHouseError>;

    async fn remove_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), IntelligentHouseError>;
}

pub struct HouseDeviceSynchronizer<T: DeviceInventory> {
    house: Arc<Mutex<IntelligentHouse>>,
    inventory: T,
}

impl<T: DeviceInventory> HouseDeviceSynchronizer<T> {
    pub fn new(house: Arc<Mutex<IntelligentHouse>>, inventory: T) -> HouseDeviceSynchronizer<T> {
        HouseDeviceSynchronizer { house, inventory }
    }
}

#[async_trait]
impl<T: DeviceInventory + Send + Sync> DeviceSynchronizer for HouseDeviceSynchronizer<T> {
    async fn add_room(&mut self, room_name: &RoomName) -> Result<(), IntelligentHouseError> {
        self.house.lock().await.add_room(room_name)?;
        self.inventory.add_room(room_name).await?;
        Ok(())
    }

    async fn remove_room(&mut self, room_name: &RoomName) -> Result<(), IntelligentHouseError> {
        self.house.lock().await.remove_room(room_name)?;
        self.inventory.remove_room(room_name).await?;
        Ok(())
    }

    async fn add_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
        device: DeviceItem,
    ) -> Result<(), IntelligentHouseError> {
        self.house.lock().await.add_device(room_name, device_name)?;
        self.inventory
            .add_device(room_name, device_name, device)
            .await?;
        Ok(())
    }

    async fn remove_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), IntelligentHouseError> {
        self.house
            .lock()
            .await
            .remove_device(room_name, device_name)?;
        self.inventory.remove_device(room_name, device_name).await?;
        Ok(())
    }
}
