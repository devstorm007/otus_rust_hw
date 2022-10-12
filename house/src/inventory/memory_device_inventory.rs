use std::collections::hash_map::Entry::Occupied;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use frunk::{hlist, Coprod};
use parking_lot::RwLock;

use crate::devices::device_info::DeviceInfo;
use crate::devices::power_socket::PowerSocket;
use crate::devices::temperature_sensor::TemperatureSensor;
use crate::errors::intelligent_house_error::InventoryError;
use crate::errors::intelligent_house_error::InventoryError::*;
use crate::house::domain::*;
use crate::inventory::device_inventory::DeviceInventory;

#[derive(Default, Clone)]
pub struct MemoryDeviceInventory {
    room_devices: Arc<RwLock<HashMap<RoomName, HashMap<DeviceName, DeviceItem>>>>,
}

pub type DeviceItem = Coprod!(PowerSocket, TemperatureSensor);

impl MemoryDeviceInventory {
    pub fn new(
        room_devices: HashMap<RoomName, HashMap<DeviceName, DeviceItem>>,
    ) -> MemoryDeviceInventory {
        MemoryDeviceInventory {
            room_devices: Arc::new(RwLock::new(room_devices)),
        }
    }
}

#[async_trait]
impl DeviceInventory for MemoryDeviceInventory {
    fn get_info(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<String, InventoryError> {
        let room_devices = self.room_devices.read();
        let device = room_devices
            .get(room_name)
            .and_then(|ds| ds.get(device_name));

        let info = device.map(|d| {
            d.fold(hlist![
                |ps: PowerSocket| ps.get_info(device_name),
                |ts: TemperatureSensor| ts.get_info(device_name)
            ])
        });

        info.ok_or_else(|| InventoryDeviceNotFound(device_name.clone(), room_name.clone()))
    }

    async fn get_rooms(&self) -> std::result::Result<Vec<RoomName>, InventoryError> {
        let room_devices = self.room_devices.read();
        Ok(room_devices.iter().map(|(k, _)| k.clone()).collect())
    }

    async fn add_room(&self, room_name: &RoomName) -> Result<(), InventoryError> {
        let mut room_devices = self.room_devices.write();
        match room_devices.get(room_name) {
            Some(_) => Err(InventoryRoomAlreadyAdded(room_name.clone())),
            None => {
                room_devices.insert(room_name.clone(), HashMap::new());
                Ok(())
            }
        }
    }

    async fn remove_room(&self, room_name: &RoomName) -> Result<(), InventoryError> {
        self.room_devices
            .write()
            .remove(room_name)
            .map(|_| ())
            .ok_or_else(|| InventoryRoomNotFound(room_name.clone()))
    }

    async fn add_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
        device: DeviceItem,
    ) -> Result<(), InventoryError> {
        let mut room_devices = self.room_devices.write();
        match room_devices.get_mut(room_name) {
            Some(devices) if !devices.contains_key(device_name) => {
                devices.insert(device_name.clone(), device);
                Ok(())
            }
            Some(_) => Err(InventoryDeviceAlreadyAdded(
                device_name.clone(),
                room_name.clone(),
            )),
            None => Err(InventoryRoomNotFound(room_name.clone())),
        }
    }

    async fn remove_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), InventoryError> {
        match self.room_devices.write().get_mut(room_name) {
            Some(devices) if devices.contains_key(device_name) => devices
                .remove(device_name)
                .map(|_| ())
                .ok_or_else(|| InventoryDeviceRemoveFailed(device_name.clone(), room_name.clone())),
            Some(_) => Err(InventoryDeviceNotFound(
                device_name.clone(),
                room_name.clone(),
            )),
            None => Err(InventoryRoomNotFound(room_name.clone())),
        }
    }

    async fn change_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
        modify: impl Fn(DeviceItem) -> Result<DeviceItem, InventoryError> + Send,
    ) -> std::result::Result<(), InventoryError> {
        match self.room_devices.write().get_mut(room_name) {
            Some(devices) => match devices.entry(device_name.clone()) {
                Occupied(mut entry) => {
                    let changed = modify(*entry.get())?;
                    entry.insert(changed);
                    Ok(())
                }
                _ => Err(InventoryDeviceNotFound(
                    device_name.clone(),
                    room_name.clone(),
                )),
            },
            None => Err(InventoryRoomNotFound(room_name.clone())),
        }
    }

    async fn get_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<DeviceItem, InventoryError> {
        let room_devices = self.room_devices.read();
        room_devices
            .get(room_name)
            .and_then(|ds| ds.get(device_name))
            .copied()
            .ok_or_else(|| InventoryDeviceNotFound(device_name.clone(), room_name.clone()))
    }
}
