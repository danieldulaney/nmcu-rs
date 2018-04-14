use errors::*;
use eventloop::{Recieve, Response};

const LIST_FILES: &'static [u8] = b"for k,v in pairs(file.list()) do print(k..\"\x1f\"..v) end";

struct File {
    name: String,
    size: usize,
}

pub struct List {
    long_mode: bool,
    files: Vec<File>,
    current: Option<usize>,
}

impl List {
    pub fn new(long_mode: bool) -> Box<Recieve> {
        Box::new(List {
            long_mode,
            files: Vec::new(),
            current: None,
        })
    }
}

impl Recieve for List {
    fn startup(&mut self) -> Response {
        let mut v = vec![0; LIST_FILES.len()];
        v.copy_from_slice(LIST_FILES);
        Response::to_serial(v)
    }

    fn recieve_serial(&mut self, payload: Vec<u8>) -> Result<Response> {
        for &b in payload.iter() {
            eprintln!("{:?}", b as char);
        }
        Ok(Response::terminate())
    }
}
