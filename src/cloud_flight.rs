use hidapi::{HidApi, HidDevice};
use log::{debug, error, info, trace};
use num::FromPrimitive;
use num_derive::FromPrimitive;
use serde::Serialize;
use std::cell::Cell;

const VENDOR_ID: u16 = 0x0951;
const PRODUCT_IDS: [u16; 1] = [0x16ea];

const BATTERY_TRIGGER_PACKET: [u8; 62] = [
    6, 0, 2, 0, 154, 0, 0, 104, 74, 142, 10, 0, 0, 0, 187, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0,
];

const CHARGE_STATUS_TRIGGER_PACKET: [u8; 62] = [
    6, 0, 2, 0, 154, 0, 0, 104, 74, 142, 10, 0, 0, 0, 187, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0,
];

const DEVICE_STATUS_TRIGGER_PACKETS: [u8; 62] = [
    6, 0, 0, 0, 0xFF, 0, 0, 104, 74, 142, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];

fn get_potential_power_event(packet: &[u8]) -> u16 {
    ((packet[1] as u16) << 8) | (packet[0] as u16)
}

fn get_potential_battery_event(packet: &[u8]) -> u16 {
    ((packet[0] as u16) << 8) | (packet[1] as u16)
}

#[derive(FromPrimitive, Debug)]
pub enum VolumeEvent {
    VolumeUp = 0x1,
    VolumeDown = 0x2,
}

#[derive(FromPrimitive, Debug)]
pub enum VolumeStateChangeEvent {
    Muted = 0x02,
    UnMuted = 0x00, // SurroundEnable = 0x13,
                    // SurroundDisable = 0x14,
}

#[derive(FromPrimitive, Debug)]
pub enum PowerEvent {
    PowerOff = 0x0301,
    PowerOn = 0x0101,
    Charging = 0x0103,
    NotCharging = 0x0003,
}

#[derive(FromPrimitive, Debug)]
pub enum BatteryEvent {
    BatteryReport = 0xbb02,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Event {
    PowerEvent(PowerEvent),
    BatteryNotificationEvent(BatteryEvent),
    VolumeEvent(VolumeEvent),
    MuteStateChangeEvent(VolumeStateChangeEvent),
    Ignored = 0xDF,
}

#[derive(FromPrimitive, Debug)]
pub enum EventType {
    VolumeLevelChange = 0x02,
    PowerStateChange = 0x0b,
    VolumeStateChange = 0x0a,
    DeviceInfoChange = 0x07,
}

#[derive(Serialize)]
pub struct CloudFlightStatus {
    powered: bool,
    muted: bool,
    surround: bool,
    charging: bool,
    battery: usize,
}

#[derive(Debug)]
pub struct CloudFlight {
    device: HidDevice,
    powered: Cell<bool>,
    muted: Cell<bool>,
    surround: Cell<bool>,
    charging: Cell<bool>,
    battery: Cell<u8>,
}

impl CloudFlight {
    pub fn new() -> Self {
        let api = HidApi::new().unwrap();

        let device = PRODUCT_IDS
            .iter()
            .map(|pid| api.open(VENDOR_ID, *pid))
            .filter(|device| device.is_ok())
            .map(|device| device.unwrap())
            .last();

        if device.is_none() {
            panic!("Not found any compatible device");
        }

        CloudFlight {
            device: device.unwrap(),
            powered: Cell::new(false),
            muted: Cell::new(false),
            surround: Cell::new(false),
            charging: Cell::new(false),
            battery: Cell::new(0),
        }
    }

    pub fn read(&self) -> Event {
        let mut buf = [0u8; 300];

        let bytes_count = self.device.read_timeout(&mut buf, 500).unwrap();

        if bytes_count > 0 && bytes_count < buf.len() {
            trace!("Read: {:02x}, {:02x?}", bytes_count, &buf[0..bytes_count]);
        } else {
            return Event::Ignored;
        }

        let event_type = if let Some(event) = EventType::from_u8(buf[0]) {
            event
        } else {
            error!("Unknown event type: {:02x}", buf[0]);

            return Event::Ignored;
        };

        match event_type {
            EventType::PowerStateChange => {
                let is_battery_event = get_potential_battery_event(&buf[2..4]);

                let battery_event = BatteryEvent::from_u16(is_battery_event);

                if battery_event.is_some() {
                    let battery_val = buf[7];

                    self.battery.set(battery_val);

                    return Event::BatteryNotificationEvent(BatteryEvent::BatteryReport);
                }

                let power_event = get_potential_power_event(&buf[3..5]);

                match PowerEvent::from_u16(power_event) {
                    Some(PowerEvent::PowerOn) => {
                        self.powered.set(true);

                        debug!("Power on");

                        Event::PowerEvent(PowerEvent::PowerOn)
                    }
                    Some(PowerEvent::PowerOff) => {
                        self.powered.set(false);

                        debug!("Power off");

                        Event::PowerEvent(PowerEvent::PowerOff)
                    }
                    Some(PowerEvent::Charging) => {
                        self.charging.set(true);

                        debug!("Cable Plugged");

                        Event::PowerEvent(PowerEvent::Charging)
                    }
                    Some(PowerEvent::NotCharging) => {
                        self.charging.set(false);

                        debug!("Cable un-plugged");

                        Event::PowerEvent(PowerEvent::NotCharging)
                    }

                    _ => Event::Ignored,
                }
            }
            EventType::DeviceInfoChange => {
                self.muted.set((buf[14] & 16) == 16);
                self.surround.set((buf[12] & 2) == 2);

                Event::Ignored
            }
            EventType::VolumeLevelChange => {
                if let Some(volume_event) = VolumeEvent::from_u8(buf[1]) {
                    return Event::VolumeEvent(volume_event);
                } else {
                    Event::Ignored
                }
            }
            EventType::VolumeStateChange => {
                // let surround = self.surround.replace((buf[2] & 2) == 2);

                // if surround != ((buf[2] & 2) == 2) {
                //     info!("SurroundSound {}", !surround);
                // }

                match VolumeStateChangeEvent::from_u8(buf[4]) {
                    Some(VolumeStateChangeEvent::Muted) => {
                        self.muted.set(true);

                        debug!("Muted");

                        Event::MuteStateChangeEvent(VolumeStateChangeEvent::Muted)
                    }
                    Some(VolumeStateChangeEvent::UnMuted) => {
                        self.muted.set(false);

                        debug!("Unmuted");

                        Event::MuteStateChangeEvent(VolumeStateChangeEvent::UnMuted)
                    }
                    None => Event::Ignored,
                }
            }
        }
    }

    pub fn check_status(&self) -> CloudFlightStatus {
        CloudFlightStatus {
            powered: self.powered.get(),
            muted: self.muted.get(),
            charging: self.charging.get(),
            surround: self.surround.get(),
            battery: self.battery.get() as usize,
        }
    }

    pub fn request_battery(&self) {
        if let Err(e) = self.device.write(&BATTERY_TRIGGER_PACKET) {
            error!("Error on requesting battery: {:?}", e);
        }

        self.read();
    }

    pub fn request_device_info(&self) {
        if let Err(e) = self.device.write(&DEVICE_STATUS_TRIGGER_PACKETS) {
            error!("Error on requesting device info: {:?}", e);
        }
    }

    pub fn request_charge_status(&self) {
        if let Err(e) = self.device.write(&CHARGE_STATUS_TRIGGER_PACKET) {
            println!("Error on requesting charge status: {:?}", e);
        }
    }
}

unsafe impl Sync for CloudFlight {}
