use thiserror::Error;

#[derive(Debug, Error)]
pub enum MurexError {
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Invalid protocol frame: {0}")]
    InvalidFrame(String),

    #[error("Unknown OpCode: {0:#04x}")]
    UnknownOpCode(u8),

    #[error("Key not found")]
    KeyNotFound,
}

pub type Key = Vec<u8>;
pub type Value = Vec<u8>;
pub type Result<T> = std::result::Result<T, MurexError>;
