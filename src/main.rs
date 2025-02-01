use clap::Command;

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

            cf.request_battery();
            cf.read();

            cf.request_charge_status();
            cf.read();

            cf.request_device_info();
            cf.read();

            loop {
                let _ = cf.read();

                let json = serde_json::to_string(&cf.check_status()).unwrap();

                println!("{}", json)
            }
        }
        _ => unreachable!(),
    }
}
