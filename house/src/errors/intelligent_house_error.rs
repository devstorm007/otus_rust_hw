use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;

use crate::errors::intelligent_house_error::HouseError::InternalError;
use crate::{DeviceName, RoomName};

#[derive(Error, Debug, Serialize)]
pub enum IntelligentHouseError {
    #[error("inventory error `{0}` raised")]
    InventoryError(#[from] InventoryError),

    #[error("house error `{0}` raised")]
    HouseError(#[from] HouseError),
}

#[derive(Error, Debug, Serialize)]
pub enum InventoryError {
    #[error("inventory device `{0}` not found")]
    InventoryDeviceNotFound(DeviceName, RoomName),

    #[error("inventory inappropriate device `{0}` for change")]
    InventoryDeviceInvalid(DeviceName, RoomName),

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

#[derive(Error, Debug, Serialize)]
pub enum HouseError {
    #[error("room `{0}` not found")]
    RoomNotFound(RoomName),

    #[error("room `{0}` already added")]
    RoomAlreadyAdded(RoomName),

    #[error("device `{0}` already added into room '{1}'")]
    RoomDeviceAlreadyAdded(DeviceName, RoomName),

    #[error("device `{0}` not found into room '{1}'")]
    RoomDeviceNotFound(DeviceName, RoomName),

    #[error("storage action failed with `{0}`")]
    InternalError(String),
}

impl HouseError {
    pub fn str<E: AsRef<str>>(err: E) -> HouseError {
        InternalError(err.as_ref().to_string())
    }

    pub fn fmt<E: Debug>(err: E) -> HouseError {
        InternalError(format!("{0:?}", err))
    }
}
