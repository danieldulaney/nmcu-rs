use std::io;
use std::os::unix::io::AsRawFd;
use std::io::{Read, Write};

use mio::{Events, Poll, PollOpt, Ready, Token, unix::EventedFd};
use mio_serial::{Serial, SerialPortSettings};

#[derive(Debug)]
pub struct Response {
    pub stdout: Option<Box<[u8]>>,
    pub serial: Option<Box<[u8]>>,
    pub terminate: bool,
}

impl Response {
    pub fn none() -> Response {
        Response {
            stdout: None,
            serial: None,
            terminate: false,
        }
    }

    pub fn to_stdout(b: Box<[u8]>) -> Response {
        Response {
            stdout: Some(b),
            serial: None,
            terminate: false,
        }
    }

    pub fn to_serial(b: Box<[u8]>) -> Response {
        Response {
            stdout: None,
            serial: Some(b),
            terminate: false,
        }
    }
}

pub trait Recieve {
    fn startup(&mut self) -> Response;
    fn recieve_stdin(&mut self, line: String) -> Result<Response, ()>;
    fn recieve_serial(&mut self, payload: Vec<u8>) -> Result<Response, ()>;
    fn shutdown(&mut self) -> Response;
}

const STDIN_TOKEN: Token = Token(0);
const SERIAL_TOKEN: Token = Token(1);

pub fn run(mut reciever: Box<Recieve>, serial_settings: &SerialPortSettings) -> usize {
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

        for event in events.iter() {
            let response = match (event.token(), event.readiness()) {
                (STDIN_TOKEN, r) if r.is_readable() => {
                    let mut line = String::new();
                    stdin.read_line(&mut line).unwrap();
                    reciever.recieve_stdin(line).unwrap()
                }
                (SERIAL_TOKEN, r) if r.is_readable() => {
                    let mut buffer = Vec::new();
                    serial.read_to_end(&mut buffer);
                    reciever.recieve_serial(buffer).unwrap()
                }
                _ => {
                    eprintln!("Unrecognized event {:?}", event);
                    Response::none()
                }
            };

            // IO flushing here is actually super important
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

    return 0;
}
