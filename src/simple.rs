// -- simplified user-friendly API for bitcore
//
// This is the RECOMMENDED API for most users. It provides:
// - Simple connection management (automatic cleanup)
// - Intuitive method names (write_str, read_line, etc.)
// - Sensible defaults with easy customization
// - Thread-safe operations without manual Arc<Mutex<>> management
//
// For advanced use cases requiring fine-grained control,
// see api.rs for the lower-level interface.

use crate::error::{BitcoreError, Result};
use crate::serial::SerialConnection;
use serialport::{DataBits, FlowControl, Parity, SerialPort, SerialPortInfo, StopBits};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info, warn};

/// simple serial connection that handles everything automatically
pub struct Serial {
    connection: Arc<Mutex<Option<SerialConnection>>>,
    config: SerialConfig,
}

/// simplified configuration for serial connections
#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub timeout: Duration,
    pub retries: usize,
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
    pub flow_control: FlowControl,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            baud_rate: 9600,
            timeout: Duration::from_secs(1),
            retries: 3,
            data_bits: DataBits::Eight,
            parity: Parity::None,
            stop_bits: StopBits::One,
            flow_control: FlowControl::None,
        }
    }
}

impl SerialConfig {
    /// create config with custom baud rate
    pub fn new(baud_rate: u32) -> Self {
        Self {
            baud_rate,
            ..Default::default()
        }
    }

    /// set timeout for operations
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// set number of retry attempts
    pub fn retries(mut self, retries: usize) -> Self {
        self.retries = retries;
        self
    }
}

impl Serial {
    /// create a new serial connection
    pub fn new<P: AsRef<str>>(port: P) -> Result<Self> {
        Self::with_config(port, SerialConfig::default())
    }

    /// create a serial connection with custom configuration
    pub fn with_config<P: AsRef<str>>(port: P, config: SerialConfig) -> Result<Self> {
        let port_builder = serialport::new(port.as_ref(), config.baud_rate)
            .data_bits(config.data_bits)
            .parity(config.parity)
            .stop_bits(config.stop_bits)
            .flow_control(config.flow_control)
            .timeout(config.timeout);

        let connection = SerialConnection::connect(port_builder)
            .map_err(|e| BitcoreError::SerialPort(e.into()))?;

        info!("connected to serial port: {}", port.as_ref());

        Ok(Self {
            connection: Arc::new(Mutex::new(Some(connection))),
            config,
        })
    }

    /// list available serial ports
    pub fn list_ports() -> Result<Vec<SerialPortInfo>> {
        SerialConnection::list().map_err(BitcoreError::Io)
    }

    /// write data to the serial port
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        if data.is_empty() {
            return Ok(0);
        }

        let mut conn_lock = self
            .connection
            .lock()
            .map_err(|e| BitcoreError::LockFailed(e.to_string()))?;

        match conn_lock.as_mut() {
            Some(conn) => {
                let mut attempts = 0;
                loop {
                    match conn.write(data) {
                        Ok(size) => {
                            debug!("wrote {} bytes", size);
                            return Ok(size);
                        }
                        Err(e) if attempts < self.config.retries => {
                            warn!("write attempt {} failed: {}", attempts + 1, e);
                            attempts += 1;
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        Err(e) => {
                            return Err(BitcoreError::Io(e));
                        }
                    }
                }
            }
            None => Err(BitcoreError::NotConnected),
        }
    }

    /// read data from the serial port
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

        let mut conn_lock = self
            .connection
            .lock()
            .map_err(|e| BitcoreError::LockFailed(e.to_string()))?;

        match conn_lock.as_mut() {
            Some(conn) => {
                // set timeout
                if let Err(e) = conn.set_timeout(self.config.timeout) {
                    warn!("failed to set timeout: {}", e);
                }

                match conn.read(buffer) {
                    Ok(bytes_read) => {
                        debug!("read {} bytes", bytes_read);
                        Ok(bytes_read)
                    }
                    Err(e) => Err(BitcoreError::Io(e)),
                }
            }
            None => Err(BitcoreError::NotConnected),
        }
    }

    /// read exact number of bytes (blocks until complete or timeout)
    pub fn read_exact(&self, buffer: &mut [u8]) -> Result<()> {
        let mut total_read = 0;
        let start_time = std::time::Instant::now();

        while total_read < buffer.len() && start_time.elapsed() < self.config.timeout {
            match self.read(&mut buffer[total_read..]) {
                Ok(0) => {
                    // no data available, continue
                    std::thread::sleep(Duration::from_millis(1));
                }
                Ok(bytes_read) => {
                    total_read += bytes_read;
                }
                Err(e) => return Err(e),
            }
        }

        if total_read == buffer.len() {
            Ok(())
        } else {
            Err(BitcoreError::Timeout {
                timeout_ms: self.config.timeout.as_millis().min(u64::MAX as u128) as u64,
            })
        }
    }

    /// write string data
    pub fn write_str(&self, data: &str) -> Result<usize> {
        self.write(data.as_bytes())
    }

    /// read into a string (until newline or timeout)
    pub fn read_line(&self) -> Result<String> {
        let mut line = String::new();
        let mut buffer = [0u8; 1];
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < self.config.timeout {
            match self.read(&mut buffer) {
                Ok(1) => {
                    let ch = buffer[0] as char;
                    if ch == '\n' {
                        break;
                    }
                    if ch != '\r' {
                        line.push(ch);
                    }
                }
                Ok(0) => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Ok(_) => {
                    // shouldn't happen with 1-byte buffer, but handle it
                    let ch = buffer[0] as char;
                    if ch == '\n' {
                        break;
                    }
                    if ch != '\r' {
                        line.push(ch);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        if line.is_empty() && start_time.elapsed() >= self.config.timeout {
            Err(BitcoreError::Timeout {
                timeout_ms: self.config.timeout.as_millis().min(u64::MAX as u128) as u64,
            })
        } else {
            Ok(line)
        }
    }

    /// flush the serial port
    pub fn flush(&self) -> Result<()> {
        let mut conn_lock = self
            .connection
            .lock()
            .map_err(|e| BitcoreError::LockFailed(e.to_string()))?;

        match conn_lock.as_mut() {
            Some(conn) => conn.flush().map_err(BitcoreError::Io),
            None => Err(BitcoreError::NotConnected),
        }
    }

    /// get port name
    pub fn port_name(&self) -> Option<String> {
        let conn_lock = self.connection.lock().ok()?;
        conn_lock.as_ref()?.name()
    }

    /// check if connected
    pub fn is_connected(&self) -> bool {
        self.connection
            .lock()
            .map(|conn| conn.is_some())
            .unwrap_or(false)
    }
}

impl Drop for Serial {
    fn drop(&mut self) {
        if let Ok(mut conn_lock) = self.connection.lock() {
            if let Some(conn) = conn_lock.take() {
                let _ = conn.disconnect();
                debug!("serial connection closed");
            }
        }
    }
}
