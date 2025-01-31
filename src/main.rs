use simple_logger::SimpleLogger;

use std::sync::Arc;

const BATTERY_REFRESH_ON_MUTE: bool = true;
const AUTO_SWITCH_TO_HEADPHONES_ON_TURN_ON: bool = true;

mod cloud_flight;

fn main() {
    SimpleLogger::new().init().unwrap();

    let cf = Arc::new(cloud_flight::CloudFlight::new());

    cf.read();

    // let mut received_device_info = false;
    // let mut received_charging_status = false;

    // loop {
    //     if !received_device_info {
    //         cf.device_info();
    //         received_device_info = true;
    //     } else if !received_charging_status {
    //         cf.charge_status();
    //         received_charging_status = true;
    //     }

    //     let event = cf.read();
    //     let mut event_happened = true;

    //     match event {
    //         cloud_flight::Event::PowerEvent(power_event) => match power_event {
    //             cloud_flight::PowerEvent::PowerOn => {
    //                 if AUTO_SWITCH_TO_HEADPHONES_ON_TURN_ON && change_to_headphones_sink() {
    //                     info!("Successfully changed to Headphones on Turn on!");
    //                 }
    //             }
    //             cloud_flight::PowerEvent::PowerOff => (),
    //             cloud_flight::PowerEvent::Charging => (),
    //             cloud_flight::PowerEvent::NotCharging => (),
    //         },
    //         cloud_flight::Event::VolumeEvent(volume_event) => match volume_event {
    //             cloud_flight::VolumeEvent::VolumeUp => (),
    //             cloud_flight::VolumeEvent::VolumeDown => (),
    //         },
    //         cloud_flight::Event::MuteStateChangeEvent(mute_state_change_event) => {
    //             match mute_state_change_event {
    //                 cloud_flight::VolumeStateChangeEvent::Muted => (),
    //                 cloud_flight::VolumeStateChangeEvent::UnMuted => (),
    //             }
    //         }

    //         cloud_flight::Event::Ignored => {
    //             event_happened = false;
    //         }
    //         cloud_flight::Event::BatteryNotificationEvent(battery_event) => match battery_event {
    //             cloud_flight::BatteryEvent::BatteryReport => {
    //                 info!("Battery: {}", value.unwrap());
    //             }
    //         }, // cloud_flight::Event::BatteryCharging => (),
    //            // cloud_flight::Event::VolumeEvent::Battery  => {
    //            //     if value.is_some(){
    //            //         info!("Battery: {}", value.unwrap());
    //            //     }
    //            // },
    //            // cloud_flight::Event::VolumeEvent::Muted => {
    //            //     if BATTERY_REFRESH_ON_MUTE {
    //            //         cf.battery();
    //            //     }
    //            // }
    //            // cloud_flight::Event::VolumeEvent::Unmuted => (),
    //            // cloud_flight::Event::VolumeEvent::PowerOff => (),
    //            // cloud_flight::Event::VolumeEvent::PowerOn => (),
    //            // cloud_flight::Event::VolumeEvent::Ignored => (),
    //     };
    // }
}
