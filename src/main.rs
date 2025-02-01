use clap::Command;

use clokwerk::{Scheduler, TimeUnits};
use cloud_flight::Event;
use simple_logger::SimpleLogger;

use std::sync::Arc;

mod cloud_flight;

fn main() {
    SimpleLogger::new().init().unwrap();

    let cli = Command::new("cloud-flight")
        .about("A simple program that interfaces with your HyperX Cloud Flight S gaming headset")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("daemon")
                .short_flag('d')
                .long_flag("daemon")
                .about("Start a daemon that shows json info about the headsets current status"),
        )
        .get_matches();

    match &cli.subcommand() {
        Some(("daemon", _)) => {
            let cf = Arc::new(cloud_flight::CloudFlight::new());

            let mut scheduler = Scheduler::new();

            let s_cf = cf.clone();
            scheduler.every(1.minutes()).run(move || {
                s_cf.request_battery();
            });

            cf.request_battery();
            cf.read();

            cf.request_charge_status();
            cf.read();

            cf.request_device_info();
            cf.read();

            loop {
                scheduler.run_pending();

                match cf.read() {
                    Event::Ignored => {}

                    _ => {
                        let json = serde_json::to_string(&cf.check_status()).unwrap();

                        println!("{}", json)
                    }
                };
            }
        }
        _ => unreachable!(),
    }
}
