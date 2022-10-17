use std::fmt::Debug;
use std::hash::Hash;

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Clone)]
pub struct HouseName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub name: RoomName,
    pub devices: Vec<DeviceName>,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Display, Serialize, Deserialize)]
pub struct RoomName(pub String);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Display, Serialize, Deserialize)]
pub struct DeviceName(pub String);
