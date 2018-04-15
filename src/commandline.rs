use errors::*;
use eventloop::{Recieve, Response};

pub struct CommandLine();

impl CommandLine {
    pub fn new() -> Box<Recieve> {
        Box::new(CommandLine())
    }
}

impl Recieve for CommandLine {
    fn recieve_stdin(&mut self, line: String) -> Result<Response> {
        Ok(Response::to_serial(line.bytes().collect::<Vec<u8>>()))
    }

    fn recieve_serial(&mut self, payload: Vec<u8>) -> Result<Response> {
        Ok(Response::to_stdout(payload))
    }
}
