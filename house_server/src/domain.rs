use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMessage {
    pub body: RequestBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestBody {
    ChangeDeviceData {
        location: DeviceLocation,
        data: DeviceData,
    },
    ShowDeviceInfo {
        location: DeviceLocation,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceLocation {
    pub room_name: String,
    pub device_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceData {
    PowerSocketState { enabled: bool },
}

/*#[derive(Debug, Serialize, Deserialize)]
pub struct PowerSocketState {
  enabled: bool,
}*/

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub body: ResponseBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseBody {
    DeviceDataChanged,
    PowerSocketInfo { enabled: bool, power: u32 },
    DeviceDescription(String),
}

/*#[derive(Debug, Serialize, Deserialize)]
pub struct PowerSocketInfo {
  pub enabled: bool,
  pub power: u32,
}
*/
/*let mut s = flexbuffers::FlexbufferSerializer::new();
let rm = RequestMessage {
  code: TypeCode::ChangePSState,
  data: vec![],
};
rm.serialize(&mut s).unwrap();
let buffer: &[u8] = s.view();*/
