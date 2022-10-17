use frunk::Coprod;

use crate::devices::power_socket::PowerSocket;
use crate::devices::temperature_sensor::TemperatureSensor;

pub type DeviceItem = Coprod!(PowerSocket, TemperatureSensor);
