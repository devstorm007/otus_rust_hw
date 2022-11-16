use frunk_core::hlist;
use futures::executor::block_on;
use house::devices::power_socket::{PowerSocket, SocketType};
use house::house::domain::{DeviceName, RoomName};
use house::inventory::device_inventory::DeviceInventory;
use house::inventory::domain::DeviceItem;
use house::inventory::memory_device_inventory::MemoryDeviceInventory;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};

#[repr(transparent)]
#[derive(Debug)]
struct InventoryHandle(*mut c_void);
impl InventoryHandle {
    pub unsafe fn as_inventory(&self) -> &'static mut MemoryDeviceInventory {
        let ptr = self.0 as *mut MemoryDeviceInventory;
        ptr.as_mut().unwrap()
    }

    pub fn from_inventory(inventory: MemoryDeviceInventory) -> Self {
        let reference = Box::leak(Box::new(inventory));
        let ptr = reference as *mut MemoryDeviceInventory;
        Self(ptr as _)
    }

    pub unsafe fn into_inventory(self) -> Box<MemoryDeviceInventory> {
        let ptr = self.0 as *mut MemoryDeviceInventory;
        Box::from_raw(ptr)
    }
}

#[repr(transparent)]
struct RawRoomName(*const c_char);
impl<'a> TryFrom<&'a RawRoomName> for RoomName {
    type Error = InventoryError;

    fn try_from(value: &'a RawRoomName) -> Result<Self, Self::Error> {
        if value.0.is_null() {
            eprintln!("RawRoomName is null");
            return Err(InventoryError::Parameter);
        }

        let s = unsafe { CStr::from_ptr(value.0) };
        let utf8_str = match s.to_str() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("RawRoomName convert failed with: {}", e);
                return Err(InventoryError::Parameter);
            }
        };

        Ok(RoomName(utf8_str.to_string()))
    }
}

#[repr(transparent)]
struct RawDeviceName(*const c_char);
impl<'a> TryFrom<&'a RawDeviceName> for DeviceName {
    type Error = InventoryError;

    fn try_from(value: &'a RawDeviceName) -> Result<Self, Self::Error> {
        if value.0.is_null() {
            eprintln!("RawDeviceName is null");
            return Err(InventoryError::Parameter);
        }

        let s = unsafe { CStr::from_ptr(value.0) };
        let utf8_str = match s.to_str() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("RawDeviceName convert failed with: {}", e);
                return Err(InventoryError::Parameter);
            }
        };

        Ok(DeviceName(utf8_str.to_string()))
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum RawEnabled {
    Disabled = 0,
    Enabled,
}

#[repr(transparent)]
struct RawSocketInfo(*const c_char);

#[repr(u8)]
enum InventoryError {
    NoError = 0,
    /*Io,
    Decoding,
    Encoding,*/
    Parameter,
    /*Unsupported,*/
    InventoryError,
}

type CreateMemoryInventoryFn = unsafe extern "C" fn(*mut InventoryHandle) -> InventoryError;
type SwitchSocketFn =
    unsafe extern "C" fn(RawRoomName, RawDeviceName, RawEnabled, InventoryHandle) -> InventoryError;
type GetSocketInfoFn =
    unsafe extern "C" fn(RawRoomName, RawDeviceName, InventoryHandle) -> RawSocketInfo;
type DestroyInventoryFn = unsafe extern "C" fn(InventoryHandle);

#[allow(unused)]
#[repr(C)]
pub struct FunctionsBlock {
    create_inventory: CreateMemoryInventoryFn,
    switch_socket: SwitchSocketFn,
    get_socket_info: GetSocketInfoFn,
    destroy_inventory: DestroyInventoryFn,
}

impl Default for FunctionsBlock {
    fn default() -> Self {
        Self {
            create_inventory,
            switch_socket,
            get_socket_info,
            destroy_inventory,
        }
    }
}

#[no_mangle]
pub extern "C" fn functions() -> FunctionsBlock {
    FunctionsBlock::default()
}

unsafe extern "C" fn create_inventory(handle: *mut InventoryHandle) -> InventoryError {
    if handle.is_null() {
        return InventoryError::Parameter;
    }

    let kitchen_name = RoomName("kitchen".to_string());

    let power_sockets = HashMap::from([(
        kitchen_name,
        HashMap::from([(
            DeviceName("socket 220V-5A".to_string()),
            DeviceItem::inject(PowerSocket {
                tpe: SocketType::C,
                voltage: 220,
                current: 5,
                enabled: true,
            }),
        )]),
    )]);

    let inventory: MemoryDeviceInventory = MemoryDeviceInventory::new(power_sockets);

    *handle = InventoryHandle::from_inventory(inventory);

    InventoryError::NoError
}

unsafe extern "C" fn switch_socket(
    room: RawRoomName,
    device: RawDeviceName,
    enabled: RawEnabled,
    handle: InventoryHandle,
) -> InventoryError {
    if handle.0.is_null() || room.0.is_null() || device.0.is_null() {
        return InventoryError::Parameter;
    }
    let room_name: RoomName = match (&room).try_into() {
        Ok(p) => p,
        Err(e) => return e,
    };
    let device_name: DeviceName = match (&device).try_into() {
        Ok(p) => p,
        Err(e) => return e,
    };
    let ps_enabled: bool = enabled == RawEnabled::Enabled;

    let inventory = handle.as_inventory();
    match block_on(inventory.change_device(&room_name, &device_name, |device| {
        device.fold(hlist![
            |mut ps: PowerSocket| {
                ps.enabled = ps_enabled;
                Ok(DeviceItem::inject(ps))
            },
            |_| Err(
                house::errors::intelligent_house_error::InventoryError::InventoryDeviceInvalid(
                    device_name.clone(),
                    room_name.clone()
                )
            )
        ])
    })) {
        Ok(_) => InventoryError::NoError,
        Err(_) => InventoryError::InventoryError,
    }
}

unsafe extern "C" fn get_socket_info(
    room: RawRoomName,
    device: RawDeviceName,
    handle: InventoryHandle,
) -> RawSocketInfo {
    if handle.0.is_null() || room.0.is_null() || device.0.is_null() {
        return mk_raw_info("error:Invalid parameters");
    }
    let room_name: &RoomName = &(match (&room).try_into() {
        Ok(p) => p,
        Err(_) => return mk_raw_info("error:Invalid room name"),
    });
    let device_name: &DeviceName = &(match (&device).try_into() {
        Ok(p) => p,
        Err(_) => return mk_raw_info("error:Invalid device name"),
    });

    let inventory = handle.as_inventory();

    match block_on(inventory.get_info(room_name, device_name)) {
        Ok(info) => mk_raw_info(info.as_ref()),
        Err(e) => mk_raw_info(format!("error:{:?}", e).as_ref()),
    }
}

fn mk_raw_info(s: &str) -> RawSocketInfo {
    if s.starts_with("error:") {
        eprintln!("get_socket_info failed with {s}");
    }
    let c_str_song = CString::new(s.as_bytes()).unwrap();
    let raw = c_str_song.into_raw();
    RawSocketInfo(raw)
}

unsafe extern "C" fn destroy_inventory(handle: InventoryHandle) {
    handle.into_inventory();
}
