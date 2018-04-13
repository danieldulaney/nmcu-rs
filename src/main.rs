extern crate mio;
extern crate mio_serial;

mod eventloop;

pub use eventloop::{Recieve, Response};

use std::time::Duration;

use mio_serial::{BaudRate, DataBits, FlowControl, Parity, SerialPortSettings, StopBits};

const SETTINGS: SerialPortSettings = SerialPortSettings {
    baud_rate: BaudRate::Baud115200,
    data_bits: DataBits::Eight,
    flow_control: FlowControl::None,
    parity: Parity::None,
    stop_bits: StopBits::One,
    timeout: Duration::from_millis(1),
};

struct CommandLine {
    last_line: String,
}

impl CommandLine {
    fn new() -> Box<CommandLine> {
        Box::new(CommandLine {
            last_line: String::new(),
        })
    }
}

impl Recieve for CommandLine {
    fn startup(&mut self) -> Response {
        Response::none()
    }

    fn recieve_stdin(&mut self, line: String) -> Result<Response, ()> {
        self.last_line = line.to_owned();

        Ok(Response::to_serial(
            line.bytes().collect::<Vec<u8>>().into_boxed_slice(),
        ))
    }

    fn recieve_serial(&mut self, payload: Vec<u8>) -> Result<Response, ()> {
        let mut print_from = 0;

        for (i, (line, &payload)) in self.last_line.bytes().zip(payload.iter()).enumerate() {
            if line == payload {
                print_from = i + 1;
            }
        }

        Ok(Response::to_stdout(
            payload
                .iter()
                .skip(print_from)
                .map(|x| *x)
                .collect::<Vec<u8>>()
                .into_boxed_slice(),
        ))
    }

    fn shutdown(&mut self) -> Response {
        Response::none()
    }
}

fn main() {
    eventloop::run(CommandLine::new(), &SETTINGS);
}
