use eventloop::Recieve;

pub struct List {}

impl List {
    pub fn new(_long_mode: bool) -> Box<Recieve> {
        Box::new(List {})
    }
}

impl Recieve for List {}
