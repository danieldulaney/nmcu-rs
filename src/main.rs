//! A serial interface for the NodeMCU firmware on ESP8266 boards.

extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate mio;
extern crate mio_serial;

pub mod errors;
pub mod eventloop;
pub mod commandline;
pub mod list;
pub mod echostrip;

pub use errors::*;
pub use eventloop::{Recieve, Response};
use commandline::CommandLine;
use list::List;

use std::time::Duration;

use mio_serial::{BaudRate, DataBits, FlowControl, Parity, SerialPortSettings, StopBits};

use clap::{App, Arg, SubCommand};

const DEFAULTS: SerialPortSettings = SerialPortSettings {
    baud_rate: BaudRate::Baud115200,
    data_bits: DataBits::Eight,
    flow_control: FlowControl::None,
    parity: Parity::None,
    stop_bits: StopBits::One,
    timeout: Duration::from_millis(1),
};

fn main() {
    let matches = App::new("nmcu")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Daniel Dulaney <dulaney.daniel@gmail.com>")
        .about("Tools for managing NodeMCU on ESP8266 chips")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PATH")
                .help("Serial port file")
                .default_value("/dev/ttyS0"),
        )
        .arg(
            Arg::with_name("baud rate")
                .short("b")
                .long("baud")
                .value_name("RATE")
                .help("Serial connection baud rate")
                .default_value("115200"),
        )
        .subcommand(SubCommand::with_name("console").about("interactive console session"))
        .subcommand(
            SubCommand::with_name("list")
                .about("list files on the device")
                .arg(Arg::with_name("long").short("l")),
        )
        .get_matches();

    let handler: Box<Recieve> = if let Some(matches) = matches.subcommand_matches("list") {
        echostrip::EchoStrip::new(List::new(matches.is_present("long")))
    } else {
        echostrip::EchoStrip::new(CommandLine::new())
    };

    eventloop::run(handler, "/dev/ttyS0", &DEFAULTS).unwrap();
}
