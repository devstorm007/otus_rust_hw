use crate::inv::bindings::{
    InventoryError, InventoryHandle, RawDeviceName, RawEnabled, RawRoomName,
};
use bindings::{Functions, FunctionsFn};
use house::house::domain::{DeviceName, RoomName};
use libloading::Library;
use std::ffi::{CStr, CString};
use std::sync::Arc;

mod bindings;

pub struct InvFactory {
    lib: Lib,
}
impl InvFactory {
    pub fn new() -> Result<Self, anyhow::Error> {
        let lib = unsafe {
            let lib = Library::new(bindings::lib_path())?;
            Lib::new(lib)?
        };

        Ok(Self { lib })
    }

    pub fn create_inventory(&self) -> Result<DeviceInventory, anyhow::Error> {
        DeviceInventory::create(self.lib.clone())
    }
}

pub struct DeviceInventory {
    lib: Lib,
    handle: InventoryHandle,
}
impl Drop for DeviceInventory {
    fn drop(&mut self) {
        unsafe {
            self.lib.destroy_inventory(self.handle);
        }
    }
}
impl DeviceInventory {
    fn create(lib: Lib) -> Result<Self, anyhow::Error> {
        let handle = unsafe { lib.create_inventory() }?;
        Ok(Self { lib, handle })
    }

    pub fn enable_socket(&mut self, rn: &RoomName, dn: &DeviceName) -> Result<(), anyhow::Error> {
        unsafe {
            Ok(self.lib.switch_socket(
                rn.try_into()?,
                dn.try_into()?,
                RawEnabled::Enabled,
                self.handle,
            )?)
        }
    }

    pub fn disable_socket(&mut self, rn: &RoomName, dn: &DeviceName) -> Result<(), anyhow::Error> {
        unsafe {
            Ok(self.lib.switch_socket(
                rn.try_into()?,
                dn.try_into()?,
                RawEnabled::Disabled,
                self.handle,
            )?)
        }
    }

    pub fn get_socket_info(&self, rn: &RoomName, dn: &DeviceName) -> Result<String, anyhow::Error> {
        unsafe {
            Ok(self
                .lib
                .get_socket_info(rn.try_into()?, dn.try_into()?, self.handle)?)
        }
    }
}

#[derive(Clone)]
#[allow(unused)]
struct Lib {
    lib: Arc<Library>,
    functions: Functions,
}
impl Lib {
    pub unsafe fn new(lib: Library) -> Result<Self, anyhow::Error> {
        let load_fn: libloading::Symbol<FunctionsFn> = lib.get(b"functions")?;
        let functions = load_fn();

        Ok(Self {
            lib: Arc::new(lib),
            functions,
        })
    }

    pub unsafe fn create_inventory(&self) -> Result<InventoryHandle, InventoryError> {
        let mut handle = InventoryHandle::new_null();

        let err = (self.functions.create_inventory)(&mut handle);
        match err {
            InventoryError::NoError => Ok(handle),
            err => Err(err),
        }
    }

    pub unsafe fn switch_socket(
        &self,
        rn: RawRoomName,
        dn: RawDeviceName,
        enabled: RawEnabled,
        handle: InventoryHandle,
    ) -> Result<(), InventoryError> {
        let err = (self.functions.switch_socket)(rn, dn, enabled, handle);
        match err {
            InventoryError::NoError => Ok(()),
            err => Err(err),
        }
    }

    pub unsafe fn get_socket_info(
        &self,
        rn: RawRoomName,
        dn: RawDeviceName,
        handle: InventoryHandle,
    ) -> Result<String, InventoryError> {
        let info = (self.functions.get_socket_info)(rn, dn, handle);

        if info.0.is_null() {
            return Err(InventoryError::NullInfo);
        }

        let s = CStr::from_ptr(info.0);
        let utf8_str = match s.to_str() {
            Ok(s) => s,
            Err(e) => {
                println!("convert info error: {:?}", e);
                return Err(InventoryError::InfoConvert);
            }
        };
        Ok(utf8_str.to_string())
    }

    pub unsafe fn destroy_inventory(&self, handle: InventoryHandle) {
        (self.functions.destroy_inventory)(handle)
    }
}

impl<'a> TryFrom<&'a RoomName> for RawRoomName {
    type Error = anyhow::Error;

    fn try_from(name: &'a RoomName) -> Result<Self, Self::Error> {
        let cs = CString::new(name.0.as_str().as_bytes())?;
        let raw = cs.into_raw();
        Ok(RawRoomName(raw))
        /*let x = rn.0.as_str();
        let rn_bytes = x.as_bytes();
        let rn_cs = CString::new(rn_bytes)?;
        let rn_ptr = rn_cs.as_ptr();
        let name = RawRoomName(rn_ptr);*/
    }
}

impl<'a> TryFrom<&'a DeviceName> for RawDeviceName {
    type Error = anyhow::Error;

    fn try_from(name: &'a DeviceName) -> Result<Self, Self::Error> {
        let cs = CString::new(name.0.as_str().as_bytes())?;
        let raw = cs.into_raw();
        Ok(RawDeviceName(raw))
        /* let rn_bytes = rn.0.as_str().as_bytes();
        let rn_cs = CString::new(rn_bytes)?;
        let rn_ptr = rn_cs.as_ptr();
        Ok(RawDeviceName(rn_ptr))*/
    }
}
