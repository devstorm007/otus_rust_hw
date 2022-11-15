use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::os::raw::c_char;
use std::path::Path;
use thiserror::Error;

#[cfg(target_os = "linux")]
pub fn lib_path() -> &'static Path {
    Path::new("target/release/libhouse_library.so")
}

#[cfg(target_os = "windows")]
pub fn lib_path() -> &'static Path {
    Path::new("target/release/libhouse_library.dll")
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct InventoryHandle(pub *const c_void);
impl InventoryHandle {
    pub unsafe fn new_null() -> Self {
        Self(std::ptr::null())
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct RawRoomName(pub *const c_char);

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct RawDeviceName(pub *const c_char);

#[repr(u8)]
#[derive(Debug)]
pub enum RawEnabled {
    Disabled = 0,
    Enabled,
}

#[repr(transparent)]
pub struct RawSocketInfo(pub *const c_char);

#[repr(u8)]
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum InventoryError {
    NoError = 0,
    Parameter,
    InternalError,
    NullInfo,
    InfoConvert,
}
impl Display for InventoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InventoryError::NoError => write!(f, "inventory no error"),
            InventoryError::Parameter => write!(f, "inventory parameter error"),
            InventoryError::InternalError => write!(f, "inventory internal error"),
            InventoryError::NullInfo => write!(f, "inventory info is null"),
            InventoryError::InfoConvert => write!(f, "inventory info failed convert"),
        }
    }
}

pub type FunctionsFn = unsafe extern "C" fn() -> Functions;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Functions {
    pub create_inventory: CreateMemoryInventoryFn,
    pub switch_socket: SwitchSocketFn,
    pub get_socket_info: GetSocketInfoFn,
    pub destroy_inventory: DestroyInventoryFn,
}

type CreateMemoryInventoryFn = unsafe extern "C" fn(*mut InventoryHandle) -> InventoryError;

type SwitchSocketFn =
    unsafe extern "C" fn(RawRoomName, RawDeviceName, RawEnabled, InventoryHandle) -> InventoryError;

type GetSocketInfoFn =
    unsafe extern "C" fn(RawRoomName, RawDeviceName, InventoryHandle) -> RawSocketInfo;

type DestroyInventoryFn = unsafe extern "C" fn(InventoryHandle);
