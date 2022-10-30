use futures::executor::block_on;
use iced::alignment::Horizontal;
use iced::{Alignment, Column, Element, Length, Sandbox, Settings, Text, Toggler};

use homework::error::GuiError;
use house::inventory::memory_device_inventory::MemoryDeviceInventory;
use house_server::domain::DeviceData::PowerSocketState;
use house_server::domain::RequestBody::{ChangeDeviceData, ShowDeviceInfo};
use house_server::domain::{DeviceLocation, RequestMessage, ResponseBody};
use house_server::house_client::HouseClient;
use house_server::house_server::HouseServer;

const TCP_SERVER_ADDRESS: &str = "127.0.0.1:45932";
const UDP_SERVER_ADDRESS: &str = "127.0.0.1:45959";

#[tokio::main]
async fn main() -> Result<(), GuiError> {
    let inventory: MemoryDeviceInventory =
        house::mk_three_rooms_inventory(house::ThreeRoomNames::default());

    HouseServer::start(inventory, TCP_SERVER_ADDRESS, UDP_SERVER_ADDRESS).await?;

    ClientGUI::run(Settings::default())?;

    Ok(())
}

struct ClientGUI {
    client: HouseClient,
    info: String,
    checked: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    On,
    Off,
}

impl Sandbox for ClientGUI {
    type Message = Message;

    fn new() -> Self {
        let client = block_on(HouseClient::connect(
            "first".to_string(),
            TCP_SERVER_ADDRESS,
            UDP_SERVER_ADDRESS,
            "127.0.0.1:41858",
        ))
        .unwrap();

        ClientGUI {
            client,
            info: Default::default(),
            checked: true,
        }
    }

    fn title(&self) -> String {
        String::from("Power socket management")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::On => {
                let response = block_on(self.client.send_and_receive(RequestMessage {
                    body: ChangeDeviceData {
                        location: DeviceLocation {
                            room_name: "kitchen".to_string(),
                            device_name: "socket4".to_string(),
                        },
                        data: PowerSocketState { enabled: true },
                    },
                }))
                .unwrap();
                match response.body {
                    ResponseBody::DeviceDataChanged => self.refresh_info(true),
                    msg => self.info = format!("failed enable: {0:?}", msg),
                };
            }
            Message::Off => {
                let response = block_on(self.client.send_and_receive(RequestMessage {
                    body: ChangeDeviceData {
                        location: DeviceLocation {
                            room_name: "kitchen".to_string(),
                            device_name: "socket4".to_string(),
                        },
                        data: PowerSocketState { enabled: false },
                    },
                }))
                .unwrap();
                match response.body {
                    ResponseBody::DeviceDataChanged => self.refresh_info(false),
                    msg => self.info = format!("failed disable: {0:?}", msg),
                };
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        self.refresh_info(self.checked);

        Column::new()
            .padding(20)
            .align_items(Alignment::Start)
            .push(
                Toggler::new(self.checked, "SWITCH ".to_string(), |b| {
                    if b {
                        Message::On
                    } else {
                        Message::Off
                    }
                })
                .width(Length::Shrink)
                .size(30)
                .text_alignment(Horizontal::Left),
            )
            .push(Text::new(&self.info).size(50))
            .into()
    }
}

impl ClientGUI {
    fn refresh_info(&mut self, checked: bool) {
        let response = block_on(self.client.send_and_receive(RequestMessage {
            body: ShowDeviceInfo {
                location: DeviceLocation {
                    room_name: "kitchen".to_string(),
                    device_name: "socket4".to_string(),
                },
            },
        }))
        .unwrap();
        match response.body {
            ResponseBody::DeviceDescription(info) => {
                self.info = info;
                self.checked = checked;
            }
            msg => self.info = format!("unexpected response: {0:?}", msg),
        };
    }
}
