use std;
use std::fmt::{self, Display};

use serde::{de, ser};
pub type DeserResult<T> = std::result::Result<T, DeserError>;

#[derive(Clone, Debug, PartialEq)]
pub enum DeserError {
    Message(String),
}

impl ser::Error for DeserError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserError::Message(msg.to_string())
    }
}

impl de::Error for DeserError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserError::Message(msg.to_string())
    }
}

impl Display for DeserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeserError::Message(msg) => formatter.write_str(msg),
        }
    }
}

impl std::error::Error for DeserError {}