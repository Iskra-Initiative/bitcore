/// bitcore - simple serial communication library
///
/// ## Quick Start
/// ```rust
/// use bitcore::Serial;
///
/// let serial = Serial::new("/dev/ttyUSB0")?;
/// serial.write_str("hello")?;
/// let response = serial.read_line()?;
/// ```
pub mod config;
pub mod error;
pub mod serial;
pub mod simple;

// main API exports
pub use error::{BitcoreError, Result};
pub use simple::{Serial, SerialConfig};

// advanced exports for power users
pub use config::RetryConfig;
