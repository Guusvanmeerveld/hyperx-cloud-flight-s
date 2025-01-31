use hidapi::{HidApi, HidDevice};
use log::{error, info};
use num::FromPrimitive;
use num_derive::FromPrimitive;
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

fn battery_percent(charge_state: u8, value: u8) -> u8 {
    match charge_state {
        0x0e => match value {
            0..=89 => 10,
            90..=119 => 15,
            120..=148 => 20,
            149..=159 => 25,
            160..=169 => 30,
            170..=179 => 35,
            180..=189 => 40,
            190..=199 => 45,
            200..=209 => 50,
            210..=219 => 55,
            220..=239 => 60,
            240..=255 => 65,
        },
        0x0f => match value {
            0..=19 => 70,
            20..=49 => 75,
            50..=69 => 80,
            70..=99 => 85,
            100..=119 => 90,
            120..=129 => 95,
            130..=255 => 100,
        },
        _ => 0,
    }
}

#[derive(FromPrimitive)]
pub enum VolumeEvent {
    VolumeUp = 0x1,
    VolumeDown = 0x2,
}

#[derive(FromPrimitive)]
pub enum VolumeStateChangeEvent {
    Muted = 0x02,
    UnMuted = 0x00, // SurroundEnable = 0x13,
                    // SurroundDisable = 0x14,
}

#[derive(FromPrimitive)]
pub enum PowerEvent {
    PowerOff = 0x0301,
    PowerOn = 0x0101,
    Charging = 0x0103,
    NotCharging = 0x0003,
}

pub enum BatteryEvent {
    BatteryReport = 0xbb02,
}

#[repr(u8)]
pub enum Event {
    PowerEvent(PowerEvent),
    BatteryNotificationEvent(BatteryEvent),
    VolumeEvent(VolumeEvent),
    MuteStateChangeEvent(VolumeStateChangeEvent),
    Ignored = 0xDF,
}

#[derive(FromPrimitive)]
pub enum EventType {
    VolumeLevelChange = 0x02,
    PowerStateChange = 0x0b,
    VolumeStateChange = 0x0a,
}

pub struct CloudFlight {
    device: HidDevice,
    pub powered: Cell<bool>,
    pub muted: Cell<bool>,
    pub surround: Cell<bool>,
    pub charging: Cell<bool>,
    pub battery: Cell<u8>,
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
            powered: Cell::new(true),
            muted: Cell::new(false),
            surround: Cell::new(false),
            charging: Cell::new(false),
            battery: Cell::new(100),
        }
    }

    pub fn read(&self) -> Event {
        let mut buf = [0u8; 300];

        let bytes = self.device.read_timeout(&mut buf, 500).unwrap();

        if bytes > 0 && bytes < buf.len() {
            info!("Read: {:02x}, {:02x?}", bytes, &buf[0..bytes]);
        } else {
            return Event::Ignored;
        }

        let event_type = if let Some(event) = EventType::from_u8(buf[0]) {
            event
        } else {
            error!("Unknown event: ${:02x}", buf[0]);

            return Event::Ignored;
        };

        let volume_event = if let Some(event) = VolumeEvent::from_u8(buf[1]) {
            event
        } else {
            return Event::Ignored;
        };

        return Event::Ignored;

        // match event {
        //     EventType::PowerStateChange => {
        //         let is_battery_event = get_potential_battery_event(&buf[2..4]);

        //         let event_type: Option<BatteryEvent> =
        //             crate::num::FromPrimitive::from_u16(is_battery_event);

        //         if event_type.is_some() {
        //             let battery_val = buf[7];
        //             self.battery.set(battery_val);
        //             let _ = optional_event_val.insert(battery_val);
        //             return (
        //                 Event::BatteryNotificationEvent(BatteryEvent::BatteryReport),
        //                 optional_event_val,
        //             );
        //         }

        //         let potential_power_event = get_potential_power_event(&buf[3..5]);

        //         let event_type: Option<PowerEvent> =
        //             crate::num::FromPrimitive::from_u16(potential_power_event);
        //         return match event_type {
        //             Some(PowerEvent::PowerOn) => {
        //                 self.battery();
        //                 self.powered.set(true);
        //                 info!("Power on");
        //                 (Event::PowerEvent(PowerEvent::PowerOn), optional_event_val)
        //             }
        //             Some(PowerEvent::PowerOff) => {
        //                 self.powered.set(false);
        //                 info!("Power off");
        //                 (Event::PowerEvent(PowerEvent::PowerOff), optional_event_val)
        //             }
        //             Some(PowerEvent::Charging) => {
        //                 info!("Cable Plugged");
        //                 self.charging.set(true);
        //                 return (Event::PowerEvent(PowerEvent::Charging), optional_event_val);
        //             }
        //             Some(PowerEvent::NotCharging) => {
        //                 info!("Cable un-plugged");
        //                 self.charging.set(false);
        //                 return (
        //                     Event::PowerEvent(PowerEvent::NotCharging),
        //                     optional_event_val,
        //                 );
        //             }
        //             None => (Event::Ignored, optional_event_val),
        //         };
        //     }
        //     EventType::VolumeLevelChange => {
        //         let event_type: Option<VolumeEvent> = crate::num::FromPrimitive::from_u8(buf[1]);
        //         match event_type {
        //             Some(VolumeEvent::VolumeUp) => {
        //                 info!("Volume up");
        //                 (
        //                     Event::VolumeEvent(VolumeEvent::VolumeUp),
        //                     optional_event_val,
        //                 )
        //             }
        //             Some(VolumeEvent::VolumeDown) => {
        //                 info!("Volume down");
        //                 (
        //                     Event::VolumeEvent(VolumeEvent::VolumeDown),
        //                     optional_event_val,
        //                 )
        //             }
        //             None => (Event::Ignored, optional_event_val),
        //         }
        //     }
        //     EventType::VolumeStateChange => {
        //         let surround = self.surround.replace((buf[2] & 2) == 2);

        //         if surround != ((buf[2] & 2) == 2) {
        //             info!("SurroundSound {}", !surround);
        //         }

        //         let event_type: Option<VolumeStateChangeEvent> =
        //             num::FromPrimitive::from_u8(buf[4]);
        //         match event_type {
        //             Some(VolumeStateChangeEvent::Muted) => {
        //                 self.muted.set(true);
        //                 info!("Muted");
        //                 (
        //                     Event::MuteStateChangeEvent(VolumeStateChangeEvent::Muted),
        //                     optional_event_val,
        //                 )
        //             }
        //             Some(VolumeStateChangeEvent::UnMuted) => {
        //                 self.muted.set(false);
        //                 info!("Unmuted");
        //                 (
        //                     Event::MuteStateChangeEvent(VolumeStateChangeEvent::UnMuted),
        //                     optional_event_val,
        //                 )
        //             }
        //             None => (Event::Ignored, optional_event_val),
        //         }
        //     }
        //     _ => Event::Ignored,
        // }
    }
    pub fn battery(&self) {
        let packet = BATTERY_TRIGGER_PACKET;
        if let Err(e) = self.device.write(&packet) {
            println!("Error on writing battery packet  {:?}", e);
        }
    }
    pub fn device_info(&self) {
        if let Err(e) = self.device.write(&DEVICE_STATUS_TRIGGER_PACKETS) {
            error!("Error on writing DEVICE_STATUS_TRIGGER_PACKET  {:?}", e);
        }

        let Some(buf) = self.read_bytes() else { return };
        self.muted.set((buf[14] & 16) == 16);
        self.surround.set((buf[12] & 2) == 2);
        info!("DeviceStart: Mute state {}", (buf[14] & 16) == 16);
        info!("DeviceStart: Surround state {}", (buf[12] & 2) == 2);
    }

    pub fn read_bytes(&self) -> Option<[u8; 300]> {
        let mut buf = [0u8; 300];
        let bytes = self.device.read_timeout(&mut buf, 500).unwrap();
        if bytes > 0 && bytes < buf.len() {
            info!("Read: {:02x}, {:02x?}", bytes, &buf[0..bytes]);
            return Some(buf);
        }
        return None;
    }
    pub fn charge_status(&self) {
        let packet = CHARGE_STATUS_TRIGGER_PACKET;
        if let Err(e) = self.device.write(&packet) {
            println!("Error on writing packet with first byte as {}, second byte as {} and third byte as {}: {:?}", packet[0], packet[1], packet[2], e);
        }
    }
}

unsafe impl Sync for CloudFlight {}
