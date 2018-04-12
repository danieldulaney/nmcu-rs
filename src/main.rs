extern crate nix;
extern crate mio;
extern crate mio_serial;

use std::io;
use std::os::unix::io::AsRawFd;
use std::time::Duration;
use std::io::{Read, Write};
use std::ascii::escape_default;
use std::ops::Deref;

use nix::sys::termios::{tcgetattr, tcsetattr, LocalFlags, Termios, SetArg};

use mio::{Events, Poll, PollOpt, Ready, Token, unix::EventedFd};
use mio_serial::{BaudRate, DataBits, FlowControl, Parity, SerialPortSettings, StopBits,
                 unix::Serial};

const SETTINGS: SerialPortSettings = SerialPortSettings {
    baud_rate: BaudRate::Baud115200,
    data_bits: DataBits::Eight,
    flow_control: FlowControl::None,
    parity: Parity::None,
    stop_bits: StopBits::One,
    timeout: Duration::from_millis(1),
};

#[derive(Debug)]
struct Response {
    stdout: Option<Box<[u8]>>,
    serial: Option<Box<[u8]>>,
    terminate: bool,
}

impl Response {
    fn none() -> Response {
        Response {
            stdout: None,
            serial: None,
            terminate: false,
        }
    }

    fn to_stdout(b: Box<[u8]>) -> Response {
        Response {
            stdout: Some(b),
            serial: None,
            terminate: false,
        }
    }

    fn to_serial(b: Box<[u8]>) -> Response {
        Response {
            stdout: None,
            serial: Some(b),
            terminate: false,
        }
    }
}

trait Recieve {
    fn startup(&mut self) -> Response;
    fn recieve_stdin(&mut self, line: &str) -> Result<Response, ()>;
    fn recieve_serial(&mut self, payload: &[u8]) -> Result<Response, ()>;
    fn shutdown(&mut self) -> Response;
}

struct CommandLine {
    original_config: Option<Termios>,
    last_line: String,
}

impl CommandLine {
    fn new() -> Box<CommandLine> {
        Box::new(CommandLine {
            original_config: None,
            last_line: String::new(),
        })
    }
}

impl Recieve for CommandLine {
    fn startup(&mut self) -> Response {
        /*
        let mut config = tcgetattr(io::stdin().as_raw_fd()).unwrap();
        self.original_config = Some(config.clone());

        config.local_flags.remove(LocalFlags::ECHO);

        tcsetattr(io::stdin().as_raw_fd(), SetArg::TCSAFLUSH, &config).unwrap();
        */

        Response::none()
    }

    fn recieve_stdin(&mut self, line: &str) -> Result<Response, ()> {
        self.last_line = line.to_owned();

        Ok(Response::to_serial(
            line.bytes().collect::<Vec<u8>>().into_boxed_slice(),
        ))
    }

    fn recieve_serial(&mut self, payload: &[u8]) -> Result<Response, ()> {

        let mut print_from = 0;
        
        for (i, (line, &payload)) in self.last_line.bytes().zip(payload.iter()).enumerate() {
            if line == payload {
                print_from = i + 1;
            }
        }

        //eprintln!("Last_line was {:?}; payload is {:?}, skipping first {}", self.last_line, payload, print_from);

        Ok(Response::to_stdout(payload.iter().skip(print_from).map(|x| *x).collect::<Vec<u8>>().into_boxed_slice()))
    }

    fn shutdown(&mut self) -> Response {
        /*
        match self.original_config {
            Some(ref c) => tcsetattr(io::stdin().as_raw_fd(), SetArg::TCSAFLUSH, c).unwrap(),
            None => {},
        }
        */

        Response::none()
    }
}

fn main() {
    event_loop(CommandLine::new(), &SETTINGS);
}

const STDIN_TOKEN: Token = Token(0);
const SERIAL_TOKEN: Token = Token(1);

fn event_loop(mut reciever: Box<Recieve>, serial_settings: &SerialPortSettings) {
    let poll = Poll::new().unwrap();

    let stdin = io::stdin();

    poll.register(
        &EventedFd(&stdin.as_raw_fd()),
        STDIN_TOKEN,
        Ready::readable(),
        PollOpt::edge(),
    ).unwrap();

    let mut serial = Serial::from_path("/dev/ttyS0", &serial_settings).unwrap();

    poll.register(&serial, SERIAL_TOKEN, Ready::readable(), PollOpt::edge())
        .unwrap();

    let mut events = Events::with_capacity(1024);

    reciever.startup();

    loop {
        poll.poll(&mut events, None).unwrap();

        //eprintln!("Top of event loop");

        for event in events.iter() {
            //eprintln!("Processing {:?}", event);

            let response = match (event.token(), event.readiness()) {
                (STDIN_TOKEN, r) if r.is_readable() => {
                    let mut line = String::new();
                    stdin.read_line(&mut line).unwrap();
                    reciever.recieve_stdin(&line).unwrap()
                }
                (SERIAL_TOKEN, r) if r.is_readable() => {
                    //let mut buffer = [0; 1];
                    //serial.read(&mut buffer);
                    let mut buffer = Vec::new();
                    serial.read_to_end(&mut buffer);
                    reciever.recieve_serial(&buffer).unwrap()
                }
                _ => {
                    println!("Unrecognized event {:?}", event);
                    Response::none()
                }
            };

            if let Some(b) = response.stdout {
                io::stdout().write(&b).unwrap();
                io::stdout().flush().unwrap();
            }

            if let Some(b) = response.serial {
                serial.write(&b).unwrap();
                serial.flush().unwrap();
            }

            if response.terminate {
                break;
            }
        }
    }

    reciever.shutdown();
}
