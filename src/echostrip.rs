use errors::*;
use eventloop::{Recieve, Response};

pub struct EchoStrip {
    destination: Box<Recieve>,
    last_line: String,
}

impl EchoStrip {
    pub fn new(destination: Box<Recieve>) -> Box<Recieve> {
        Box::new(EchoStrip {
            destination,
            last_line: String::new(),
        })
    }
}

impl Recieve for EchoStrip {
    fn recieve_stdin(&mut self, line: String) -> Result<Response> {
        self.last_line = line.clone();

        self.destination.recieve_stdin(line)
    }

    fn recieve_serial(&mut self, payload: Vec<u8>) -> Result<Response> {
        let mut line_index = 0;
        let mut payload_index = 0;

        {
            let line = self.last_line.as_bytes();

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
        self.last_line = String::new();

        // Skip the first payload_index bytes
        self.destination.recieve_serial(
            payload
                .iter()
                .skip(payload_index)
                .map(|x| *x)
                .collect::<Vec<u8>>(),
        )
    }
}
