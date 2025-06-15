// -- performance benchmarks for bitcore simplified API
// run with: cargo bench

use bitcore::{config::RetryConfig, Serial, SerialConfig};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
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
            return Err(
                "socat not found in PATH. Please install socat to run these benchmarks.".into(),
            );
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

/// create a connection with standard benchmark settings
fn create_benchmark_connection(port: &str) -> Result<Serial, Box<dyn std::error::Error>> {
    let config = SerialConfig::new(115200) // use higher baud rate for benchmarks
        .timeout(Duration::from_millis(100))
        .retries(3);

    let serial = Serial::with_config(port, config)?;
    Ok(serial)
}

fn benchmark_retry_config_creation(c: &mut Criterion) {
    c.bench_function("retry_config_creation", |b| {
        b.iter(|| {
            black_box(
                RetryConfig::new(5)
                    .with_delay(Duration::from_millis(100))
                    .with_backoff(2.0),
            )
        })
    });
}

fn benchmark_retry_config_delay_calculation(c: &mut Criterion) {
    let config = RetryConfig::new(10)
        .with_delay(Duration::from_millis(50))
        .with_backoff(1.5);

    c.bench_function("retry_config_delay_calculation", |b| {
        b.iter(|| {
            for attempt in 0..10 {
                black_box(config.delay_for_attempt(attempt));
            }
        })
    });
}

fn benchmark_serial_config_creation(c: &mut Criterion) {
    c.bench_function("serial_config_creation", |b| {
        b.iter(|| {
            black_box(
                SerialConfig::new(115200)
                    .timeout(Duration::from_millis(500))
                    .retries(5),
            )
        })
    });
}

fn benchmark_serial_config_builder(c: &mut Criterion) {
    c.bench_function("serial_config_builder", |b| {
        b.iter(|| {
            black_box(
                SerialConfig::default()
                    .timeout(Duration::from_millis(100))
                    .retries(3),
            )
        })
    });
}

fn benchmark_connection_creation(c: &mut Criterion) {
    c.bench_function("connection_creation", |b| {
        b.iter(|| {
            // benchmark just the config creation since actual connection requires hardware
            black_box(SerialConfig::new(9600))
        })
    });
}

fn benchmark_core_transmit_receive(c: &mut Criterion) {
    // try to create socat manager, skip if not available
    let socat = match SocatManager::new() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("skipping core transmit/receive benchmarks: socat not available");
            return;
        }
    };

    let conn1 = create_benchmark_connection(socat.port1()).expect("failed to connect to port1");
    let conn2 = create_benchmark_connection(socat.port2()).expect("failed to connect to port2");

    // benchmark core transmit functionality
    c.bench_function("core_transmit", |b| {
        let test_data = b"core transmit test data";
        b.iter(|| {
            black_box(conn1.write(test_data).expect("transmit failed"));
        })
    });

    // benchmark core receive functionality
    c.bench_function("core_receive", |b| {
        let test_data = b"core receive test data";

        b.iter(|| {
            // write data first
            conn1.write(test_data).expect("write failed");

            // then receive it
            let mut buffer = [0u8; 64];
            black_box(conn2.read(&mut buffer).expect("receive failed"));
        })
    });

    // benchmark transmit/receive round-trip performance
    c.bench_function("core_transmit_receive_roundtrip", |b| {
        let test_data = b"roundtrip";

        b.iter(|| {
            let start = Instant::now();

            // transmit
            conn1.write(test_data).expect("transmit failed");

            // receive
            let mut buffer = [0u8; 64];
            conn2.read(&mut buffer).expect("receive failed");

            black_box(start.elapsed());
        })
    });

    // benchmark different data sizes for transmit/receive
    let mut size_group = c.benchmark_group("core_transmit_receive_sizes");

    for size in [1, 8, 32, 128, 512, 1024].iter() {
        let data = vec![0x42u8; *size];

        size_group.throughput(Throughput::Bytes(*size as u64));
        size_group.bench_with_input(BenchmarkId::new("transmit_receive", size), size, |b, _| {
            b.iter(|| {
                // transmit
                conn1.write(&data).expect("transmit failed");

                // receive
                let mut buffer = vec![0u8; *size + 64]; // extra space for safety
                let bytes_read = conn2.read(&mut buffer).expect("receive failed");
                black_box(bytes_read);
            })
        });
    }
    size_group.finish();

    // automatic cleanup on drop
}

fn benchmark_error_handling(c: &mut Criterion) {
    let test_data = b"benchmark test data";

    c.bench_function("error_handling_write", |b| {
        b.iter(|| {
            // this will always fail since no connection exists
            let result = Serial::new("/dev/nonexistent_port_for_benchmark");
            match result {
                Ok(serial) => {
                    let _ = black_box(serial.write(test_data));
                }
                Err(_) => {
                    // expected error
                    black_box(());
                }
            }
        })
    });

    c.bench_function("error_handling_read", |b| {
        b.iter(|| {
            // this will always fail since no connection exists
            let result = Serial::new("/dev/nonexistent_port_for_benchmark");
            match result {
                Ok(serial) => {
                    let mut buffer = [0u8; 64];
                    let _ = black_box(serial.read(&mut buffer));
                }
                Err(_) => {
                    // expected error
                    black_box(());
                }
            }
        })
    });
}

fn benchmark_serial_operations(c: &mut Criterion) {
    // try to create socat manager, skip if not available
    let socat = match SocatManager::new() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("skipping serial benchmarks: socat not available");
            return;
        }
    };

    // benchmark connection establishment
    c.bench_function("serial_connect_disconnect", |b| {
        b.iter(|| {
            let connection = create_benchmark_connection(socat.port1()).expect("failed to connect");
            black_box(&connection);
            // automatic cleanup on drop
        })
    });

    // setup connections for read/write benchmarks
    let conn1 = create_benchmark_connection(socat.port1()).expect("failed to connect to port1");
    let conn2 = create_benchmark_connection(socat.port2()).expect("failed to connect to port2");

    // benchmark small writes
    let small_data = b"hello";
    c.bench_function("serial_write_small", |b| {
        b.iter(|| {
            black_box(conn1.write(small_data).expect("write failed"));
        })
    });

    // benchmark medium writes
    let medium_data = vec![0x55u8; 256]; // 256 bytes
    c.bench_function("serial_write_medium", |b| {
        b.iter(|| {
            black_box(conn1.write(&medium_data).expect("write failed"));
        })
    });

    // benchmark large writes
    let large_data = vec![0xAAu8; 4096]; // 4KB
    c.bench_function("serial_write_large", |b| {
        b.iter(|| {
            black_box(conn1.write(&large_data).expect("write failed"));
        })
    });

    // benchmark throughput with different data sizes
    let mut throughput_group = c.benchmark_group("serial_throughput");

    for size in [64, 256, 1024, 4096].iter() {
        let data = vec![0x42u8; *size];

        throughput_group.throughput(Throughput::Bytes(*size as u64));
        throughput_group.bench_with_input(BenchmarkId::new("write", size), size, |b, _| {
            b.iter(|| {
                black_box(conn1.write(&data).expect("write failed"));
            })
        });
    }
    throughput_group.finish();

    // benchmark read operations
    c.bench_function("serial_read_with_data", |b| {
        // pre-fill the buffer by writing data
        let test_data = b"benchmark read data";

        b.iter(|| {
            // write data first
            conn1.write(test_data).expect("write failed");

            // then read it
            let mut buffer = [0u8; 64];
            black_box(conn2.read(&mut buffer).expect("read failed"));
        })
    });

    // benchmark read timeout (when no data available)
    c.bench_function("serial_read_timeout", |b| {
        // create a connection with very short timeout for this benchmark
        let timeout_config = SerialConfig::new(115200)
            .timeout(Duration::from_millis(1))
            .retries(1);
        let timeout_conn =
            Serial::with_config(socat.port2(), timeout_config).expect("failed to connect");

        b.iter(|| {
            let mut buffer = [0u8; 64];
            // this should timeout quickly
            let _ = black_box(timeout_conn.read(&mut buffer));
        })
    });

    // benchmark flush operation
    c.bench_function("serial_flush", |b| {
        b.iter(|| {
            conn1.flush().expect("flush failed");
            black_box(());
        })
    });

    // benchmark round-trip latency
    c.bench_function("serial_round_trip", |b| {
        let ping_data = b"ping";

        b.iter(|| {
            let start = Instant::now();

            // write ping
            conn1.write(ping_data).expect("write failed");

            // read response
            let mut buffer = [0u8; 64];
            conn2.read(&mut buffer).expect("read failed");

            black_box(start.elapsed());
        })
    });

    // automatic cleanup on drop
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    // try to create socat manager, skip if not available
    let socat = match SocatManager::new() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("skipping concurrent benchmarks: socat not available");
            return;
        }
    };

    let conn1 =
        Arc::new(create_benchmark_connection(socat.port1()).expect("failed to connect to port1"));
    let conn2 =
        Arc::new(create_benchmark_connection(socat.port2()).expect("failed to connect to port2"));

    // benchmark concurrent read/write operations
    c.bench_function("serial_concurrent_readwrite", |b| {
        b.iter(|| {
            let conn1_clone = Arc::clone(&conn1);
            let conn2_clone = Arc::clone(&conn2);

            let writer = thread::spawn(move || {
                let data = b"concurrent test";
                for _ in 0..10 {
                    let _ = conn1_clone.write(data);
                }
            });

            let reader = thread::spawn(move || {
                let mut buffer = [0u8; 64];
                for _ in 0..10 {
                    let _ = conn2_clone.read(&mut buffer);
                }
            });

            writer.join().unwrap();
            reader.join().unwrap();
        })
    });

    // automatic cleanup on drop
}

criterion_group!(
    config_benches,
    benchmark_retry_config_creation,
    benchmark_retry_config_delay_calculation,
    benchmark_serial_config_creation,
    benchmark_serial_config_builder,
    benchmark_connection_creation,
    benchmark_error_handling
);

criterion_group!(
    serial_benches,
    benchmark_serial_operations,
    benchmark_concurrent_operations
);

criterion_group!(core_benches, benchmark_core_transmit_receive);

criterion_main!(config_benches, serial_benches, core_benches);
