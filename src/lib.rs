pub mod config;
pub mod error;
pub mod serial;
pub mod simple;

// main API exports
pub use error::{BitcoreError, Result};
pub use simple::{Serial, SerialConfig};

// advanced exports for power users
pub use config::RetryConfig;
