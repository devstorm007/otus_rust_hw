use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::string::ToString;

use async_trait::async_trait;
use frunk_core::hlist;
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::Database;

use house::devices::power_socket::PowerSocket;
use house::devices::temperature_sensor::TemperatureSensor;
use house::errors::intelligent_house_error::InventoryError;
use house::errors::intelligent_house_error::InventoryError::{
    InventoryDeviceAlreadyAdded, InventoryDeviceNameDuplicated, InventoryDeviceNotFound,
    InventoryInternalError, InventoryRoomNotFound,
};
use house::house::domain::{DeviceName, RoomName};
use house::inventory::device_inventory::DeviceInventory;
use house::inventory::domain::{get_info, DeviceItem, RoomDevices};

#[derive(Clone)]
pub struct DbDeviceInventory {
    db: Database,
}

impl DbDeviceInventory {
    pub fn new(db: Database) -> DbDeviceInventory {
        DbDeviceInventory { db }
    }

    async fn get_room_devices(&self, room_name: &RoomName) -> Result<RoomDevices, InventoryError> {
        self.db
            .collection::<RoomDevices>(ROOM_DEVICES_TABLE)
            .find_one(doc! {"name": room_name.0.as_str()}, None)
            .await
            .map_err(InventoryError::fmt)?
            .ok_or_else(|| InventoryRoomNotFound(room_name.clone()))
    }

    async fn save_devices(
        &self,
        room_name: &RoomName,
        room_devices: RoomDevices,
    ) -> Result<(), InventoryError> {
        let sockets_doc =
            mongodb::bson::to_document(&room_devices.sockets).map_err(InventoryError::fmt)?;
        let sensors_doc =
            mongodb::bson::to_document(&room_devices.sensors).map_err(InventoryError::fmt)?;

        self.db
            .collection::<RoomDevices>(ROOM_DEVICES_TABLE)
            .update_one(
                doc! {"name": room_name.0.as_str()},
                doc! {"$set": { "sockets": sockets_doc, "sensors": sensors_doc }},
                None,
            )
            .await
            .map(|_| ())
            .map_err(InventoryError::fmt)
    }
}

#[async_trait]
impl DeviceInventory for DbDeviceInventory {
    async fn get_info(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<String, InventoryError> {
        let device = self.get_device(room_name, device_name).await?;

        Ok(get_info(device, device_name))
    }

    async fn get_rooms(&self) -> Result<Vec<RoomName>, InventoryError> {
        let room_devices = self.get_all_room_devices().await?;
        let room_names = room_devices.iter().map(|rd| rd.name.clone()).collect();

        Ok(room_names)
    }

    async fn add_room(&self, room_name: &RoomName) -> Result<(), InventoryError> {
        let room = RoomDevices {
            name: room_name.clone(),
            ..Default::default()
        };

        self.db
            .collection::<RoomDevices>(ROOM_DEVICES_TABLE)
            .insert_one(room, None)
            .await
            .map_err(InventoryError::fmt)?;

        Ok(())
    }

    async fn remove_room(&self, room_name: &RoomName) -> Result<(), InventoryError> {
        self.db
            .collection::<RoomDevices>(ROOM_DEVICES_TABLE)
            .delete_one(doc! {"name": room_name.0.as_str()}, None)
            .await
            .map_err(InventoryError::fmt)?;

        Ok(())
    }

    async fn get_all_room_devices(&self) -> Result<Vec<RoomDevices>, InventoryError> {
        let cursor = self
            .db
            .collection::<RoomDevices>(ROOM_DEVICES_TABLE)
            .find(None, None)
            .await
            .map_err(InventoryError::fmt)?;

        cursor.try_collect().await.map_err(InventoryError::fmt)
    }

    async fn add_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
        device: DeviceItem,
    ) -> Result<(), InventoryError> {
        let mut room_devices = self.get_room_devices(room_name).await?;

        match (
            room_devices.sockets.get(device_name),
            room_devices.sensors.get(device_name),
        ) {
            (None, None) => {
                device.fold(hlist![
                    |ps: PowerSocket| {
                        room_devices.sockets.insert(device_name.clone(), ps);
                    },
                    |ts: TemperatureSensor| {
                        room_devices.sensors.insert(device_name.clone(), ts);
                    }
                ]);
                Ok(())
            }
            (_, _) => Err(InventoryDeviceAlreadyAdded(
                device_name.clone(),
                room_name.clone(),
            )),
        }?;

        self.save_devices(room_name, room_devices).await
    }

    async fn remove_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), InventoryError> {
        let mut room_devices = self.get_room_devices(room_name).await?;

        room_devices.sockets.remove(device_name);
        room_devices.sensors.remove(device_name);

        self.save_devices(room_name, room_devices).await
    }

    async fn change_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
        modify: impl Fn(DeviceItem) -> Result<DeviceItem, InventoryError> + Send,
    ) -> Result<(), InventoryError> {
        let mut room_devices = self.get_room_devices(room_name).await?;

        match (
            room_devices.sockets.entry(device_name.clone()),
            room_devices.sensors.entry(device_name.clone()),
        ) {
            (Occupied(_), Occupied(_)) => Err(InventoryDeviceNameDuplicated(
                device_name.clone(),
                room_name.clone(),
            )),
            (Vacant(_), Vacant(_)) => Err(InventoryDeviceNotFound(
                device_name.clone(),
                room_name.clone(),
            )),
            (Occupied(mut socket_entry), _) => {
                let item = DeviceItem::inject(*socket_entry.get());
                let changed = modify(item)?;
                let data = changed
                    .get()
                    .ok_or_else(|| InventoryInternalError("socket is empty".to_string()))?;
                socket_entry.insert(*data);
                Ok(())
            }
            (_, Occupied(mut sensor_entry)) => {
                let item = DeviceItem::inject(*sensor_entry.get());
                let changed = modify(item)?;
                let data = changed
                    .get()
                    .ok_or_else(|| InventoryInternalError("sensor is empty".to_string()))?;
                sensor_entry.insert(*data);
                Ok(())
            }
        }?;

        self.save_devices(room_name, room_devices).await
    }

    async fn get_device(
        &self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<DeviceItem, InventoryError> {
        let room_devices = self.get_room_devices(room_name).await?;

        match (
            room_devices.sockets.get(device_name),
            room_devices.sensors.get(device_name),
        ) {
            (Some(_), Some(_)) => Err(InventoryDeviceNameDuplicated(
                device_name.clone(),
                room_name.clone(),
            )),
            (None, None) => Err(InventoryDeviceNotFound(
                device_name.clone(),
                room_name.clone(),
            )),
            (Some(socket), None) => Ok(DeviceItem::inject(*socket)),
            (None, Some(sensor)) => Ok(DeviceItem::inject(*sensor)),
        }
    }
}

const ROOM_DEVICES_TABLE: &str = "room_devices";
