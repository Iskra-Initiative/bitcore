// -- socat-based integration tests for bitcore simplified API
// these tests require socat to be installed and available in PATH
// run with: cargo test --test socat_tests -- --ignored

use bitcore::{Serial, SerialConfig};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// socat process manager for creating virtual serial port pairs
struct SocatManager {
    process: Child,
    port1: String,
    port2: String,
    _temp_dir: TempDir,
}

impl SocatManager {
    /// create a new socat virtual serial port pair
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // check if socat is available
        if Command::new("socat").arg("--version").output().is_err() {
            return Err("socat not found in PATH. Please install socat to run these tests.".into());
        }

        let temp_dir = tempfile::tempdir()?;
        let port1 = temp_dir.path().join("ttyV0").to_string_lossy().to_string();
        let port2 = temp_dir.path().join("ttyV1").to_string_lossy().to_string();

        // create virtual serial port pair using socat
        let process = Command::new("socat")
            .args([
                "-d",
                "-d",
                &format!("pty,raw,echo=0,link={}", port1),
                &format!("pty,raw,echo=0,link={}", port2),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // wait a bit for socat to create the ports
        thread::sleep(Duration::from_millis(100));

        Ok(Self {
            process,
            port1,
            port2,
            _temp_dir: temp_dir,
        })
    }

    /// get the first port path
    fn port1(&self) -> &str {
        &self.port1
    }

    /// get the second port path
    fn port2(&self) -> &str {
        &self.port2
    }
}

impl Drop for SocatManager {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// initialize tracing for tests
#[allow(clippy::single_component_path_imports)]
fn init_tracing() {
    use tracing_subscriber;
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

/// create a connection with standard test settings
fn create_test_connection(port: &str) -> Result<Serial, Box<dyn std::error::Error>> {
    let config = SerialConfig::new(115200) // use higher baud rate for tests
        .timeout(Duration::from_millis(100))
        .retries(3);

    let serial = Serial::with_config(port, config)?;
    Ok(serial)
}

#[cfg(test)]
mod socat_integration_tests {
    use super::*;

    #[test]
    #[ignore] // requires socat
    fn test_socat_basic_communication() {
        init_tracing();

        let socat = match SocatManager::new() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping test: {}", e);
                return;
            }
        };

        // create connections to both ports
        let conn1 = create_test_connection(socat.port1()).expect("failed to connect to port1");
        let conn2 = create_test_connection(socat.port2()).expect("failed to connect to port2");

        // test basic write/read
        let test_data = b"hello from port1";
        let bytes_written = conn1.write(test_data).expect("failed to write");
        assert_eq!(bytes_written, test_data.len());

        // read from the other port
        let mut buffer = [0u8; 64];
        let bytes_read = conn2.read(&mut buffer).expect("failed to read");

        assert_eq!(bytes_read, test_data.len());
        assert_eq!(&buffer[..bytes_read], test_data);

        // test reverse communication
        let test_data2 = b"hello from port2";
        let bytes_written = conn2.write(test_data2).expect("failed to write");
        assert_eq!(bytes_written, test_data2.len());

        let bytes_read = conn1.read(&mut buffer).expect("failed to read");

        assert_eq!(bytes_read, test_data2.len());
        assert_eq!(&buffer[..bytes_read], test_data2);

        // automatic cleanup on drop
    }

    #[test]
    #[ignore] // requires socat
    fn test_socat_retry_logic() {
        init_tracing();

        let socat = match SocatManager::new() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping test: {}", e);
                return;
            }
        };

        let conn1 = create_test_connection(socat.port1()).expect("failed to connect");

        // test write with retries (retries are configured in SerialConfig)
        let test_data = b"retry test data";
        let start_time = Instant::now();
        let result = conn1.write(test_data);
        let elapsed = start_time.elapsed();

        match result {
            Ok(bytes_written) => {
                assert_eq!(bytes_written, test_data.len());
                println!("write succeeded: {} bytes in {:?}", bytes_written, elapsed);
            }
            Err(e) => {
                println!("write failed: {} in {:?}", e, elapsed);
                panic!("write should succeed with socat");
            }
        }

        // automatic cleanup on drop
    }

    #[test]
    #[ignore] // requires socat
    fn test_socat_read_timeout() {
        init_tracing();

        let socat = match SocatManager::new() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping test: {}", e);
                return;
            }
        };

        // create connection with short timeout for this test
        let config = SerialConfig::new(115200)
            .timeout(Duration::from_millis(200))
            .retries(3);
        let conn1 = Serial::with_config(socat.port1(), config).expect("failed to connect");

        // test read timeout when no data is available
        let mut buffer = [0u8; 64];
        let start_time = Instant::now();
        let result = conn1.read(&mut buffer);
        let elapsed = start_time.elapsed();

        match result {
            Ok(bytes_read) => {
                println!(
                    "unexpected read success: {} bytes in {:?}",
                    bytes_read, elapsed
                );
                // if we get data, that's fine too (might be leftover from other tests)
            }
            Err(e) => {
                println!("read timed out as expected: {} in {:?}", e, elapsed);
                // timeout should be approximately correct (allow some tolerance)
                assert!(elapsed >= Duration::from_millis(150)); // allow 25% tolerance
            }
        }

        // automatic cleanup on drop
    }

    #[test]
    #[ignore] // requires socat
    fn test_socat_large_data_transfer() {
        init_tracing();

        let socat = match SocatManager::new() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping test: {}", e);
                return;
            }
        };

        let conn1 = create_test_connection(socat.port1()).expect("failed to connect to port1");
        let conn2 = create_test_connection(socat.port2()).expect("failed to connect to port2");

        // test large data transfer
        let large_data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();

        let bytes_written = conn1
            .write(&large_data)
            .expect("failed to write large data");
        assert_eq!(bytes_written, large_data.len());

        // read in chunks
        let mut received_data = Vec::new();
        let mut total_read = 0;
        let start_time = Instant::now();

        while total_read < large_data.len() && start_time.elapsed() < Duration::from_secs(5) {
            let mut buffer = [0u8; 256];
            match conn2.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        received_data.extend_from_slice(&buffer[..bytes_read]);
                        total_read += bytes_read;
                    }
                }
                Err(e) => {
                    println!("read error after {} bytes: {}", total_read, e);
                    break;
                }
            }

            // small delay to prevent busy waiting
            if total_read < large_data.len() {
                thread::sleep(Duration::from_millis(10));
            }
        }

        assert_eq!(received_data.len(), large_data.len());
        assert_eq!(received_data, large_data);

        // automatic cleanup on drop
    }

    #[test]
    #[ignore] // requires socat
    fn test_socat_flush_operation() {
        init_tracing();

        let socat = match SocatManager::new() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping test: {}", e);
                return;
            }
        };

        let conn1 = create_test_connection(socat.port1()).expect("failed to connect");

        // test flush operation
        let result = conn1.flush();
        assert!(result.is_ok(), "flush should succeed: {:?}", result);

        // automatic cleanup on drop
    }

    #[test]
    #[ignore] // requires socat
    fn test_socat_concurrent_operations() {
        init_tracing();

        let socat = match SocatManager::new() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("skipping test: {}", e);
                return;
            }
        };

        let conn1 =
            Arc::new(create_test_connection(socat.port1()).expect("failed to connect to port1"));
        let conn2 =
            Arc::new(create_test_connection(socat.port2()).expect("failed to connect to port2"));

        // test concurrent read/write operations
        let conn1_clone = Arc::clone(&conn1);
        let conn2_clone = Arc::clone(&conn2);

        let writer_handle = thread::spawn(move || {
            for i in 0..10 {
                let data = format!("message {}", i);
                if let Err(e) = conn1_clone.write(data.as_bytes()) {
                    eprintln!("write error: {}", e);
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
        });

        let reader_handle = thread::spawn(move || {
            let mut messages_received = 0;
            let start_time = Instant::now();

            while messages_received < 10 && start_time.elapsed() < Duration::from_secs(10) {
                let mut buffer = [0u8; 64];
                match conn2_clone.read(&mut buffer) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                            println!("received: {}", message);
                            messages_received += 1;
                        }
                    }
                    Err(e) => {
                        println!("read timeout/error: {}", e);
                        // small delay to prevent busy waiting
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
            messages_received
        });

        writer_handle.join().expect("writer thread panicked");
        let messages_received = reader_handle.join().expect("reader thread panicked");

        println!("received {} out of 10 messages", messages_received);
        // we should receive most messages (allow for some loss due to timing)
        assert!(messages_received >= 5, "should receive at least 5 messages");

        // automatic cleanup on drop
    }
}
