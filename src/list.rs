use errors::*;
use eventloop::{Recieve, Response};

pub struct List {
    long_mode: bool,
}

impl Recieve for List {}
