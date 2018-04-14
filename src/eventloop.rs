use errors::*;

use std::io;
use std::os::unix::io::AsRawFd;
use std::io::{ErrorKind::WouldBlock, Read, Write};

use mio::{Events, Poll, PollOpt, Ready, Token, unix::EventedFd};
use mio_serial::{Serial, SerialPortSettings};

#[derive(Debug)]
pub struct Response {
    pub stdout: Option<Vec<u8>>,
    pub serial: Option<Vec<u8>>,
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

    pub fn to_stdout(b: Vec<u8>) -> Response {
        Response {
            stdout: Some(b),
            serial: None,
            terminate: false,
        }
    }

    pub fn to_serial(b: Vec<u8>) -> Response {
        Response {
            stdout: None,
            serial: Some(b),
            terminate: false,
        }
    }

    pub fn terminate() -> Response {
        Response {
            stdout: None,
            serial: None,
            terminate: true,
        }
    }
}

pub trait Recieve {
    fn startup(&mut self) -> Response {
        Response::none()
    }

    fn recieve_stdin(&mut self, _line: String) -> Result<Response> {
        Ok(Response::none())
    }

    fn recieve_serial(&mut self, _payload: Vec<u8>) -> Result<Response> {
        Ok(Response::none())
    }

    fn shutdown(&mut self) -> Response {
        Response::none()
    }
}

const STDIN_TOKEN: Token = Token(0);
const SERIAL_TOKEN: Token = Token(1);

pub fn run(
    mut reciever: Box<Recieve>,
    serial_path: &str,
    serial_settings: &SerialPortSettings,
) -> Result<()> {
    let poll = Poll::new().unwrap();

    poll.register(
        &EventedFd(&io::stdin().as_raw_fd()),
        STDIN_TOKEN,
        Ready::readable(),
        PollOpt::edge(),
    ).unwrap();

    // Serial has to be mutable because Read and Write need mutability
    let mut serial = Serial::from_path(serial_path, &serial_settings).unwrap();

    poll.register(&serial, SERIAL_TOKEN, Ready::readable(), PollOpt::edge())
        .unwrap();

    // 1024 is arbitrary (it's what the docs used :) ); we'll probably never have more than 1 or 2
    let mut events = Events::with_capacity(1024);

    // Dispatch this *before* the loop
    let startup_response = reciever.startup();

    write_response(&startup_response, &mut serial);

    if !startup_response.terminate {
        'main: loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                // Read in the event and dispatch it to the Reciever, storing the response
                let response = match (event.token(), event.readiness()) {
                    (STDIN_TOKEN, r) if r.is_readable() => {
                        // This assumes that get a line at a time from stdin
                        let mut line = String::new();
                        io::stdin().read_line(&mut line)?;

                        // Dispatch it to the receiver
                        reciever.recieve_stdin(line)?
                    }
                    (SERIAL_TOKEN, r) if r.is_readable() => {
                        let mut buffer = Vec::new();

                        // Ignore return value and WouldBlock errors
                        // WouldBlock is returned when we run out of input, which is normal
                        match serial.read_to_end(&mut buffer) {
                            Ok(_) => {}
                            Err(ref e) if e.kind() == WouldBlock => {}
                            r => {
                                r?;
                            }
                        };

                        // Dispatch it to the receiver
                        reciever.recieve_serial(buffer)?
                    }
                    _ => {
                        // Spurious events are probably possible in some edge cases
                        eprintln!("Unrecognized event {:?}", event);
                        Response::none()
                    }
                };

                write_response(&response, &mut serial)?;

                if response.terminate {
                    eprintln!("Terminating");
                    break 'main;
                }
            }
        }
    }

    // Dispatch this *after* the loop
    write_response(&reciever.shutdown(), &mut serial);

    // Just return void, but without any errors!
    Ok(())
}

fn write_response(response: &Response, serial: &mut Serial) -> Result<()> {
    // IO flushing here is super important to keep everything in sync
    if let Some(ref b) = response.stdout {
        io::stdout().write(b).unwrap();
        io::stdout().flush().unwrap();
    }

    if let Some(ref b) = response.serial {
        serial.write(b)?;
        serial.flush()?;
    }

    Ok(())
}
