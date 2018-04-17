use errors::*;
use eventloop::{Recieve, Response};

pub struct EchoStrip {
    destination: Box<Recieve>,
    last_line: Vec<u8>,
}

impl EchoStrip {
    pub fn new(destination: Box<Recieve>) -> Box<Recieve> {
        Box::new(EchoStrip {
            destination,
            last_line: Vec::new(),
        })
    }

    pub fn save_response(&mut self, response: &Response) {
        if let Some(ref data) = response.serial {
            self.last_line = data.clone();
        }
    }

    pub fn try_save_response(&mut self, response: Result<Response>) -> Result<Response> {
        if let Ok(ref r) = response {
            self.save_response(r);
        }

        response
    }
}

impl Recieve for EchoStrip {
    fn recieve_stdin(&mut self, line: String) -> Result<Response> {
        let r = self.destination.recieve_stdin(line);
        self.try_save_response(r)
    }

    fn recieve_serial(&mut self, payload: Vec<u8>) -> Result<Response> {
        let mut line_index = 0;
        let mut payload_index = 0;

        {
            let line = &self.last_line[..];

            loop {
                if let (Some(&line_cur), Some(&payload_cur)) =
                    (line.get(line_index), payload.get(payload_index))
                {
                    if line_cur == payload_cur {
                        // They match, consume one of each
                        line_index += 1;
                        payload_index += 1;
                    } else if line_cur == '\n' as u8 && payload_cur == '\r' as u8
                    // The line looks like "...\n", while the payload looks like "...\r\n"
                    // Consume one of the line and two of the payload
                    && line.get(line_index + 1) == None
                        && payload.get(payload_index + 1) == Some(&('\n' as u8))
                    {
                        line_index += 1;
                        payload_index += 2;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // You only ever get one chance to match a given line
        self.last_line = Vec::new();

        // Skip the first payload_index bytes
        let r = self.destination.recieve_serial(
            payload
                .iter()
                .skip(payload_index)
                .map(|x| *x)
                .collect::<Vec<u8>>(),
        );

        self.try_save_response(r)
    }

    fn startup(&mut self) -> Response {
        let r = self.destination.startup();

        self.save_response(&r);

        r
    }

    fn shutdown(&mut self) -> Response {
        let r = self.destination.shutdown();

        self.save_response(&r);

        r
    }
}
