use crate::{DeviceName, RoomName};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("inventory error `{0}` raised")]
    InventoryError(#[from] InventoryError),

    #[error("house error `{0}` raised")]
    HouseError(#[from] HouseError),
}

#[derive(Error, Debug)]
pub enum InventoryError {
    #[error("inventory device `{0}` not found")]
    InventoryDeviceNotFound(DeviceName, RoomName),

    #[error("inventory device `{0}` already added into room {1}")]
    InventoryDeviceAlreadyAdded(DeviceName, RoomName),

    #[error("inventory device `{0}` add failed")]
    InventoryDeviceAddFailed(DeviceName, RoomName),

    #[error("inventory device `{0}` remove failed")]
    InventoryDeviceRemoveFailed(DeviceName, RoomName),

    #[error("inventory room `{0}` with devices not found")]
    InventoryRoomNotFound(RoomName),

    #[error("inventory room `{0}` already added")]
    InventoryRoomAlreadyAdded(RoomName),
}

#[derive(Error, Debug)]
pub enum HouseError {
    #[error("room `{0}` not found")]
    RoomNotFound(RoomName),

    #[error("room `{0}` already added")]
    RoomAlreadyAdded(RoomName),

    #[error("device `{0}` already added into room '{1}'")]
    RoomDeviceAlreadyAdded(DeviceName, RoomName),

    #[error("device `{0}` not found into room '{1}'")]
    RoomDeviceNotFound(DeviceName, RoomName),
}
