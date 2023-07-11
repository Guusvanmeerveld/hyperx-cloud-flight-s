use std::env;
use std::path::{Path};
use ksni::menu::StandardItem;
use ksni::{Icon, ToolTip};
use std::sync::Arc;
use std::vec::Vec;

use crate::cloud_flight;

const HEADPHONES_MUTED: &str = "audio-headphones-black";
const HEADPHONES_BATTERY_CHARGING: &str = "battery-060-charging";
const HEADPHONES_BATTERY_FULL: &str = "audio-headset-black";
const HEADPHONES_BATTERY_GOOD: &str = "audio-headset-black";
const HEADPHONES_BATTERY_MEDIUM: &str = "audio-headset-black";
const HEADPHONES_BATTERY_LOW: &str = "audio-headset-black";
const HEADPHONES_BATTERY_CAUTION: &str = "battery-010";
const HEADPHONES_BATTERY_EMPTY: &str = "battery-empty.svg";

pub struct Tray {
    cf: Arc<cloud_flight::CloudFlight>,
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "hyperx".to_string()
    }

    fn icon_theme_path(&self) -> String {
        let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") else {return "".parse().unwrap() };
        Path::new(&manifest_dir).join("assets").into_os_string().into_string().unwrap_or_default()
    }

    fn icon_name(&self) -> String {
        return if self.cf.muted.get() {
            if self.cf.surround.get() {
                HEADPHONES_MUTED.to_string() + ("-surround.svg")
            } else {
                HEADPHONES_MUTED.to_string() + ".svg"
            }
        } else if self.cf.charging.get() {
            HEADPHONES_BATTERY_CHARGING.to_string() +".svg"
        } else {
            let mut res: String;
            match self.cf.battery.get() {
                0..=19 => {
                    res = HEADPHONES_BATTERY_CAUTION.parse().unwrap();
                }
                20..=39 => {
                    res = HEADPHONES_BATTERY_LOW.parse().unwrap();
                }
                40..=59 => {
                    res = HEADPHONES_BATTERY_MEDIUM.parse().unwrap();
                }
                60..=89 => {
                    res = HEADPHONES_BATTERY_GOOD.parse().unwrap();
                }
                90..=100 => {
                    res = HEADPHONES_BATTERY_FULL.parse().unwrap();
                }
                _ => {
                    res = HEADPHONES_BATTERY_EMPTY.parse().unwrap();
                }
            }
            if self.cf.surround.get() {
                res += "-surround";
            }
            res += ".svg";
            res
        }
    }

    fn tool_tip(&self) -> ToolTip {
        let description: String;
        if self.cf.charging.get() {
            description = format!("Charging battery");
        } else {
            description = format!("Battery: {}%", self.cf.battery.get());
        }
        ToolTip {
            title: "HyperX Cloud Flight".into(),
            description,
            icon_name: "".into(),
            icon_pixmap: Vec::new(),
        }
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let muted_text: String;
        if self.cf.muted.get() {
            muted_text = "Yes".into();
        } else {
            muted_text = "No".into();
        }
        let surround_test: String;
        if self.cf.surround.get() {
            surround_test = "On".into();
        } else {
            surround_test = "Off".into();
        }


        let battery_text: String;
        if self.cf.charging.get() {
            battery_text = "Battery charging".into();
        } else {
            battery_text = format!("Battery level: {}", self.cf.battery.get());
        }

        vec![
            StandardItem {
                label: "HyperX Cloud Flight".into(),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: format!("Muted: {}", muted_text),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: format!("Surround 7.1: {}", surround_test),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: battery_text,
                activate: {
                    let cf = self.cf.clone();
                    Box::new(move |_| cf.clone().battery())
                },
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub struct TrayService {
    handle: ksni::Handle<Tray>,

}

impl TrayService {
    pub fn new(cf: Arc<cloud_flight::CloudFlight>) -> Self {

        let svc = ksni::TrayService::new(Tray { cf });

        let handle = svc.handle();

        svc.spawn();
        TrayService { handle }
    }

    pub fn update(&self) {
        self.handle.update(|_: &mut Tray| {});
    }
}
