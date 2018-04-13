use errors::*;
use eventloop::{Recieve, Response};

struct File {
    name: String,
    size: usize,
}

pub struct List {
    long_mode: bool,
    files: Vec<File>,
}

impl List {
    pub fn new(long_mode: bool) -> Box<Recieve> {
        Box::new(List {
            long_mode,
            files: Vec::new(),
        })
    }
}

impl Recieve for List {
    fn startup(&mut self) -> Response {
        Response::terminate()
    }
}
