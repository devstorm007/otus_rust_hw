use derive_more::From;
use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;

use crate::errors::intelligent_house_error::HouseError::HouseInternalError;
use crate::errors::intelligent_house_error::InventoryError::InventoryInternalError;
use crate::{DeviceName, RoomName};

#[derive(Error, Debug, Serialize, From)]
pub enum IntelligentHouseError {
    #[error("inventory error `{0}` raised")]
    InventoryErr(InventoryError),

    #[error("house error `{0}` raised")]
    HouseErr(HouseError),
}

#[derive(Error, Debug, Serialize)]
pub enum InventoryError {
    #[error("inventory device `{0}` not found into room {1}")]
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

    #[error("inventory device name `{0}` duplicated into room {1}")]
    InventoryDeviceNameDuplicated(DeviceName, RoomName),

    #[error("inventory action failed with `{0}`")]
    InventoryInternalError(String),
}

impl InventoryError {
    pub fn str<E: AsRef<str>>(err: E) -> InventoryError {
        InventoryInternalError(err.as_ref().to_string())
    }

    pub fn fmt<E: Debug>(err: E) -> InventoryError {
        InventoryInternalError(format!("{0:?}", err))
    }
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
    HouseInternalError(String),
}

impl HouseError {
    pub fn str<E: AsRef<str>>(err: E) -> HouseError {
        HouseInternalError(err.as_ref().to_string())
    }

    pub fn fmt<E: Debug>(err: E) -> HouseError {
        HouseInternalError(format!("{0:?}", err))
    }
}
