// -- error handling for bitcore

use core::fmt;
use std::io;

/// custom error type for bitcore operations
#[derive(Debug)]
pub enum BitcoreError {
    /// serial port error
    SerialPort(serialport::Error),

    /// io error
    Io(io::Error),

    /// connection not established
    NotConnected,

    /// connection already exists
    AlreadyConnected,

    /// lock acquisition failed
    LockFailed(String),

    /// operation timed out
    Timeout { timeout_ms: u64 },

    /// retry limit exceeded
    RetryLimitExceeded { attempts: usize },

    /// invalid parameter
    InvalidParameter { param: String, reason: String },
}

impl fmt::Display for BitcoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitcoreError::SerialPort(e) => write!(f, "serial port error: {e}"),
            BitcoreError::Io(e) => write!(f, "io error: {e}"),
            BitcoreError::NotConnected => write!(f, "connection not established"),
            BitcoreError::AlreadyConnected => write!(f, "connection already exists"),
            BitcoreError::LockFailed(msg) => write!(f, "lock acquisition failed: {msg}"),
            BitcoreError::Timeout { timeout_ms } => {
                write!(f, "operation timed out after {timeout_ms}ms")
            }
            BitcoreError::RetryLimitExceeded { attempts } => {
                write!(f, "retry limit exceeded: {attempts} attempts failed")
            }
            BitcoreError::InvalidParameter { param, reason } => {
                write!(f, "invalid parameter {param}: {reason}")
            }
        }
    }
}

impl std::error::Error for BitcoreError {}

impl From<serialport::Error> for BitcoreError {
    fn from(err: serialport::Error) -> Self {
        BitcoreError::SerialPort(err)
    }
}

impl From<io::Error> for BitcoreError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotConnected => BitcoreError::NotConnected,
            io::ErrorKind::TimedOut => BitcoreError::Timeout { timeout_ms: 0 },
            io::ErrorKind::AlreadyExists => BitcoreError::AlreadyConnected,
            // Keep Io() for less common I/O errors like UnexpectedEof, WriteZero, etc.
            _ => BitcoreError::Io(err),
        }
    }
}

impl From<BitcoreError> for io::Error {
    fn from(err: BitcoreError) -> Self {
        match err {
            BitcoreError::Io(io_err) => io_err,
            BitcoreError::NotConnected => io::Error::new(io::ErrorKind::NotConnected, err),
            BitcoreError::Timeout { .. } => io::Error::new(io::ErrorKind::TimedOut, err),
            _ => io::Error::other(err),
        }
    }
}

/// result type alias for bitcore operations
pub type Result<T> = core::result::Result<T, BitcoreError>;
