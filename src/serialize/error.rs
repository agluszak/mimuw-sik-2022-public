use std::fmt::Display;

use serde::{de, ser};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, DeserError>;

#[derive(Debug, Error)]
pub enum DeserError {
    #[error("Invalid enum variant: {0}")]
    InvalidEnumVariant(u8),
    #[error("Expected unit (byte '0')")]
    InvalidUnit,
    #[error("Expected option (byte '0' or '1')")]
    InvalidOption,
    #[error("Expected boolean (byte '0' or '1')")]
    InvalidBool,
    #[error("Length of the sequence must be known")]
    UnknownLength,
    #[error("Not all bytes were processed")]
    TrailingData,
    #[error("Encoding failed: {0}")]
    Encoding(String),
    #[error("Decoding failed: {0}")]
    Decoding(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Integer conversion error: {0}")]
    Int(#[from] std::num::TryFromIntError),
}

impl ser::Error for DeserError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserError::Encoding(msg.to_string())
    }
}

impl de::Error for DeserError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserError::Decoding(msg.to_string())
    }
}
