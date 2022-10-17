use std::fmt::Debug;

use async_trait::async_trait;
use futures::stream::StreamExt;
use futures::{stream, TryStreamExt};
use mongodb::bson::doc;
use mongodb::Database;

use house::errors::intelligent_house_error::HouseError;
use house::errors::intelligent_house_error::HouseError::RoomNotFound;
use house::house::domain::{DeviceName, HouseName, Room, RoomName};
use house::house::intelligent_house::IntelligentHouse;
use house::inventory::device_inventory::DeviceInventory;

#[derive(Debug, Clone)]
pub struct DbIntelligentHouse {
    name: HouseName,
    db: Database,
}

impl DbIntelligentHouse {
    pub fn new(name: &str, db: Database) -> Self {
        DbIntelligentHouse {
            name: HouseName(name.to_string()),
            db,
        }
    }
}

const ROOMS_TABLE: &str = "rooms";

#[async_trait]
impl IntelligentHouse for DbIntelligentHouse {
    async fn get_rooms(&self) -> Result<Vec<Room>, HouseError> {
        let rooms = self
            .db
            .collection::<Room>(ROOMS_TABLE)
            .find(None, None)
            .await
            .map_err(HouseError::fmt)?;

        rooms.try_collect().await.map_err(HouseError::fmt)
    }

    async fn get_room(&self, room_name: &RoomName) -> Result<Room, HouseError> {
        let room_opt = self
            .db
            .collection::<Room>("rooms")
            .find_one(doc! {"name": room_name.0.as_str()}, None)
            .await
            .map_err(HouseError::fmt)?;

        room_opt.ok_or_else(|| RoomNotFound(room_name.clone()))
    }

    async fn add_room(&mut self, room_name: &RoomName) -> Result<(), HouseError> {
        let room = Room {
            name: room_name.clone(),
            devices: vec![],
        };

        self.db
            .collection::<Room>(ROOMS_TABLE)
            .insert_one(room, None)
            .await
            .map_err(HouseError::fmt)?;

        Ok(())
    }

    async fn remove_room(&mut self, room_name: &RoomName) -> Result<(), HouseError> {
        self.db
            .collection::<Room>(ROOMS_TABLE)
            .delete_one(doc! {"name": room_name.0.as_str()}, None)
            .await
            .map_err(HouseError::fmt)?;

        Ok(())
    }

    async fn get_devices(&self, room_name: &RoomName) -> Result<Vec<DeviceName>, HouseError> {
        let room = self.get_room(room_name).await?;

        Ok(room.devices)
    }

    async fn add_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), HouseError> {
        let mut room = self.get_room(room_name).await?;
        room.devices.push(device_name.clone());
        let devices_doc = mongodb::bson::to_document(&room.devices).map_err(HouseError::fmt)?;

        self.db
            .collection::<Room>(ROOMS_TABLE)
            .update_one(
                doc! {"name": room_name.0.as_str()},
                doc! {"$set": { "devices": devices_doc }},
                None,
            )
            .await
            .map_err(HouseError::fmt)?;

        Ok(())
    }

    async fn remove_device(
        &mut self,
        room_name: &RoomName,
        device_name: &DeviceName,
    ) -> Result<(), HouseError> {
        let room = self.get_room(room_name).await?;
        let filtered: Vec<&DeviceName> = room
            .devices
            .iter()
            .filter(|d| d.0 != device_name.0)
            .collect();

        let devices_doc = mongodb::bson::to_document(&filtered).unwrap();

        self.db
            .collection::<Room>(ROOMS_TABLE)
            .update_one(
                doc! {"name": room_name.0.as_str()},
                doc! {"$set": { "devices": devices_doc }},
                None,
            )
            .await
            .map_err(HouseError::fmt)?;

        Ok(())
    }

    async fn generate_report<T: DeviceInventory + Sync>(
        &self,
        inventory: &T,
    ) -> Result<String, HouseError> {
        let rooms = self.get_rooms().await?;
        let prefix_msg = format!("'{}' contains {} rooms:\n", self.name.0, rooms.len());

        stream::iter(rooms)
            .fold(Ok(prefix_msg), |house_info, room| async move {
                let (devices_info, _) = stream::iter(room.devices)
                    .fold(
                        ("".to_string(), room.name.clone()),
                        |(acc_dev_info, rn), device_name| async move {
                            let _dev_info: String = inventory
                                .get_info(&rn, &device_name)
                                .await
                                .unwrap_or_else(|e| format!("{e}"));

                            (format!("{acc_dev_info}     {_dev_info}\n"), rn)
                        },
                    )
                    .await;

                Ok(format!(
                    "{}   '{}' has:\n{}\n",
                    house_info?, room.name.0, devices_info
                ))
            })
            .await
    }
}
