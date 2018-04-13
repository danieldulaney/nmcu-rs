//! A serial interface for the NodeMCU firmware on ESP8266 boards.

#[macro_use]
extern crate error_chain;
extern crate mio;
extern crate mio_serial;

pub mod errors;
pub mod eventloop;
pub mod commandline;

pub use errors::*;
pub use eventloop::{Recieve, Response};

use std::time::Duration;

use mio_serial::{BaudRate, DataBits, FlowControl, Parity, SerialPortSettings, StopBits};

const DEFAULTS: SerialPortSettings = SerialPortSettings {
    baud_rate: BaudRate::Baud115200,
    data_bits: DataBits::Eight,
    flow_control: FlowControl::None,
    parity: Parity::None,
    stop_bits: StopBits::One,
    timeout: Duration::from_millis(1),
};

fn main() {
    eventloop::run(commandline::CommandLine::new(), "/dev/ttyS0", &DEFAULTS).unwrap();
}
