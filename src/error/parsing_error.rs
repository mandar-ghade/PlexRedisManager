use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ServerGroupParsingError {
    msg: String,
}

impl Display for ServerGroupParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl Error for ServerGroupParsingError {}

impl ServerGroupParsingError {
    pub fn new(msg: String) -> Self {
        ServerGroupParsingError { msg }
    }
}
