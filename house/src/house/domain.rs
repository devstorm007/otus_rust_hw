use std::fmt::Debug;
use std::hash::Hash;

use derive_more::Display;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Room {
    pub name: RoomName,
    pub devices: Vec<DeviceName>,
}

#[derive(Debug, Display, Clone)]
pub struct HouseName(pub String);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Display, Serialize)]
pub struct RoomName(pub String);

#[derive(Eq, PartialEq, Hash, Debug, Clone, Display, Serialize)]
pub struct DeviceName(pub String);
