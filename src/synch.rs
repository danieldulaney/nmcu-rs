use errors::*;
use eventloop::{Recieve, Response};

use std::mem;

static PROMPT: &'static [u8] = b"\r\n> ";

/// Stay in sync with the NodeMCU command line
///
/// The NodeMCU command line always starts its prompts with `\r\n> `. This
/// interface will send `\n` until it detects that it has recieved a valid
/// NodeMCU command line at the end of a response, and only then pass data on
/// to the child.
pub struct Synch {
    inner: Box<Recieve>,
    state: WaitingFor,
}

#[derive(Debug)]
enum WaitingFor {
    FirstSynch,
    Nothing,
    Response,
    Synch(Response),
    Invalid,
}

impl Synch {
    pub fn new(inner: Box<Recieve>) -> Box<Recieve> {
        Box::new(Synch {
            inner,
            state: WaitingFor::FirstSynch,
        })
    }

    pub fn start_synched(inner: Box<Recieve>) -> Box<Recieve> {
        Box::new(Synch {
            inner,
            state: WaitingFor::Nothing,
        })
    }

    /// Determine if the response has a serial component.
    /// If it does, go into `WaitingFor::Response`. If it doesn't go into
    /// `WaitingFor::Nothing`, then pass it out unchanged.
    fn decide_response(&mut self, resp: Result<Response>) -> Result<Response> {
        if let Ok(ref r) = resp {
            match r.serial {
                Some(_) => self.state = WaitingFor::Response,
                None => self.state = WaitingFor::Nothing,
            }
        }

        resp
    }
}

impl Recieve for Synch {
    fn startup(&mut self) -> Response {
        debug!("Synch startup with state {:?}", self.state);

        match self.state {
            WaitingFor::FirstSynch => response_endline(),
            WaitingFor::Nothing => {
                let resp = self.inner.startup();
                self.decide_response(Ok(resp)).expect("Passed in Ok")
            }
            _ => unreachable!("Synch must start off waiting for FirstSynch or Nothing"),
        }
    }

    fn recieve_serial(&mut self, payload: Vec<u8>) -> Result<Response> {
        match (&mut self.state, ends_with(&payload, PROMPT)) {
            (&mut WaitingFor::FirstSynch, false) | (&mut WaitingFor::Synch(_), false) => {
                Ok(response_endline())
            }

            (state @ &mut WaitingFor::FirstSynch, true) => {
                let resp = self.inner.startup();
                *state = match resp.serial {
                    Some(_) => WaitingFor::Response,
                    None => WaitingFor::Nothing,
                };

                Ok(resp)
            }

            (state @ &mut WaitingFor::Nothing, _) | (state @ &mut WaitingFor::Response, true) => {
                let r = self.inner.recieve_serial(payload);

                if let Ok(ref resp) = r {
                    *state = match resp.serial {
                        Some(_) => WaitingFor::Response,
                        None => WaitingFor::Nothing,
                    };
                }

                r
            }

            (state @ &mut WaitingFor::Synch(_), true) => {
                let mut temp_state = WaitingFor::Invalid;

                mem::swap(state, &mut temp_state);

                if let WaitingFor::Synch(resp) = temp_state {
                    *state = match resp.serial {
                        Some(_) => WaitingFor::Response,
                        None => WaitingFor::Nothing,
                    };

                    Ok(resp)
                } else {
                    unreachable!("temp_state must be WaitingFor::Synch");
                }
            }

            (state @ &mut WaitingFor::Response, false) => {
                *state = WaitingFor::Synch(self.inner.recieve_serial(payload)?);

                Ok(response_endline())
            }

            (&mut WaitingFor::Invalid, _) => {
                unreachable!("Synch should never be left in an invalid state")
            }
        }
    }

    fn recieve_stdin(&mut self, _payload: String) -> Result<Response> {
        unimplemented!()
    }

    fn shutdown(&mut self) -> Response {
        self.inner.shutdown()
    }
}

fn response_endline() -> Response {
    Response::to_serial(vec![';' as u8, '\r' as u8, '\n' as u8])
}

fn ends_with(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }

    let (_, tail) = haystack.split_at(haystack.len() - needle.len());

    tail.eq(needle)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ends_with() {
        assert!(ends_with(b"abcdefg", b"efg"));
        assert!(!ends_with(b"abcdefg", b"efh"));
        assert!(!ends_with(b"defg", b"abcdefg"));
    }
}
