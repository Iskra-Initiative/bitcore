// -- higher level API that lib.rs exposes
// provides user-facing API, wraps the lower-level routines in a more convenient interface
// note: this module  handles:
// - thread safety
// - shared state
// - buffered read/write operations
// - connection validation
// - logging
// - error handling
// - retry logic
// without cluttering the core (serial.rs)) logic
// note: input validation is to be handled by the frontend

use crate::serial::SerialConnection;
use serialport::{SerialPortBuilder, SerialPortInfo};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex, MutexGuard};

/// mutex-protected shared connection object
pub type SharedConnection = Arc<Mutex<Option<SerialConnection>>>;

use log::{error, info, warn};
use std::time::{Duration, Instant}; // Make sure to add `log` to your Cargo.toml

/// lock a shared connection
fn lock_connection(
    connection: &SharedConnection,
) -> io::Result<MutexGuard<Option<SerialConnection>>> {
    connection.lock().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("[core] failure to acquire lock: {}", e),
        )
    })
}

/// list available serial ports
///
/// @return io::Result<Vec<SerialPortInfo>> - result of the operation
pub fn list() -> io::Result<Vec<SerialPortInfo>> {
    SerialConnection::list()
}

/// connect to a serial port
///
/// @param shared_conn: &SharedConnection - shared connection object
/// @param port: SerialPortBuilder - serial port builder object
///
/// @return io::Result<()> - result of the operation
pub fn connect(shared_conn: &SharedConnection, port: SerialPortBuilder) -> io::Result<()> {
    // validate specific connection properties here
    //
    //

    info!("[core] connecting to {:?}", port);

    let conn_result = SerialConnection::connect(port).map_err(|e| {
        io::Error::new(
            io::ErrorKind::ConnectionRefused,
            format!("[core] connection refused: {}", e),
        )
    });

    match conn_result {
        Ok(conn) => {
            let mut conn_lock = lock_connection(shared_conn)?;
            *conn_lock = Some(conn);
            info!("[core] connected");
            Ok(())
        }
        Err(e) => {
            error!("[core] connection failure: {}", e);
            Err(e)
        }
    }
}

/// disconnect from a serial port
///
/// @param shared_conn: &SharedConnection - shared connection object
///
/// @return io::Result<()> - result of the operation
pub fn disconnect(shared_conn: &SharedConnection) -> io::Result<()> {
    let mut conn_lock = lock_connection(shared_conn)?;
    match conn_lock.take() {
        Some(conn) => {
            info!("[core] disconnecting");
            conn.disconnect()
        }
        None => {
            warn!("[core] lock not obtained (likely not connected)");
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "[core] no connection",
            ))
        }
    }
}

/// write data to a serial port
///
/// @param shared_conn: &SharedConnection - shared connection object
/// @param data: &[u8] - data to write
///
/// @return io::Result<usize> - result of the operation
pub fn write(shared_conn: &SharedConnection, data: &[u8], retries: usize) -> io::Result<usize> {
    // validate input parameters here
    //
    //

    let mut conn_lock = lock_connection(shared_conn)?;

    match conn_lock.as_mut() {
        Some(conn) => {
            let mut attempts = 0;
            loop {
                match conn.write(data) {
                    Ok(size) => {
                        info!("[core] wrote {} b", size);
                        return Ok(size);
                    }
                    Err(ref _e) if attempts < retries => {
                        warn!("[core] write failure #{}", attempts + 1);
                        attempts += 1;
                    }
                    Err(e) => {
                        error!("[core] write failed after {} attempts: {}", retries, e);
                        return Err(e);
                    }
                }
            }
        }
        None => {
            warn!("[core] attempted write on a non-existing connection");
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "[core] no connection",
            ))
        }
    }
}

/// read data from a serial port
///
/// @param shared_conn: &SharedConnection - shared connection object
/// @param buffer: &mut [u8] - buffer to read data into
/// @param timeout: Duration - read timeout
///
/// @return io::Result<usize> - result of the operation
pub fn read(
    shared_conn: &SharedConnection,
    buffer: &mut [u8],
    timeout: Duration,
) -> io::Result<usize> {
    let mut conn_lock = lock_connection(shared_conn)?;

    match conn_lock.as_mut() {
        Some(conn) => {
            info!("[core] reading data with timeout of {:?}", timeout);
            let start_time = Instant::now();
            match conn.read(buffer) {
                Ok(size) => {
                    info!("[core] read {} b", size);
                    Ok(size)
                }
                Err(e) if start_time.elapsed() < timeout => {
                    warn!("[core] read interrupted, retrying...");

                    // retry logic here

                    Err(e)
                }
                Err(e) => {
                    error!("[core] read failed after timeout: {}", e);
                    Err(e)
                }
            }
        }
        None => {
            warn!("[core] attempted read on a non-existing connection");
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "[core] no connection",
            ))
        }
    }
}

/// flush serial port
///
/// @param shared_conn: &SharedConnection - shared connection object
///
/// @return io::Result<()> - result of the operation
pub fn flush(shared_conn: &SharedConnection) -> io::Result<()> {
    let mut conn_lock = lock_connection(shared_conn)?;

    match conn_lock.as_mut() {
        Some(conn) => {
            info!("[core] flushing");
            conn.flush()
        }
        None => {
            warn!("[core] attempted flush on a non-existing connection");
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "[core] no connection",
            ))
        }
    }
}
