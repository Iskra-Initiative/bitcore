# bitcore

[![Rust CI](https://github.com/yourusername/bitcore/workflows/Rust%20CI/badge.svg)](https://github.com/yourusername/bitcore/actions)
[![Coverage](https://img.shields.io/badge/coverage-0%25-red)](https://github.com/yourusername/bitcore/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**Simple, fast, and reliable serial communication for Rust.**

## ✨ Features

- **🚀 Simple API** - Connect and communicate in just 2 lines of code
- **🔒 Thread-safe** - Safe concurrent access with automatic connection management
- **⚡ Fast** - Optimized polling and minimal overhead
- **🔧 Configurable** - Flexible retry logic and timeouts
- **🧪 Well-tested** - Comprehensive test suite with socat integration
- **📦 No_std compatible** - Works in embedded environments

## 🚀 Quick Start

```rust
use bitcore::Serial;

// connect and use - that's it!
let serial = Serial::new("/dev/ttyUSB0")?;
serial.write_str("hello world")?;
let response = serial.read_line()?;
```

## 📖 Examples

### Basic Usage
```rust
use bitcore::{Serial, SerialConfig};
use std::time::Duration;

// simple connection with defaults (9600 baud, 1s timeout, 3 retries)
let serial = Serial::new("/dev/ttyUSB0")?;

// write data
serial.write(b"hello")?;
serial.write_str("world\n")?;

// read data
let mut buffer = [0u8; 64];
let bytes_read = serial.read(&mut buffer)?;
let line = serial.read_line()?;  // read until newline
```

### Custom Configuration
```rust
let config = SerialConfig::new(115200)
    .timeout(Duration::from_millis(500))
    .retries(5);

let serial = Serial::with_config("/dev/ttyUSB0", config)?;
```

### List Available Ports

```rust
for port in Serial::list_ports()? {
    println!("Found port: {}", port.port_name);
}
```

## 🧪 Testing

```bash
# unit tests (no hardware required)
cargo test

# integration tests with socat (requires: sudo apt install socat)
cargo test --test socat_tests -- --ignored

# run example
cargo run --example basic_usage
```

## 📊 Performance

Run benchmarks to see performance metrics:

```bash
cargo bench                    # all benchmarks
cargo bench config_benches     # configuration benchmarks
cargo bench serial_benches     # serial communication benchmarks (requires socat)
```

## 🔧 Advanced Usage

For complex scenarios, you can still use the lower-level API:

```rust
use bitcore::{connect, disconnect, read, write, SharedConnection};
use std::sync::{Arc, Mutex};

let connection: SharedConnection = Arc::new(Mutex::new(None));
let port_builder = serialport::new("/dev/ttyUSB0", 9600);
connect(&connection, port_builder)?;
write(&connection, b"data", 3)?;  // 3 retries
disconnect(&connection)?;
```

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

- **TODO**
  - admin:
    - add license
    - add documentation (Vocs / Nextra / GitBook)
  - tech:
    - buffered write/read
    - connection validation ('heart beat')
    - extend api with an action loop default implementation (callable by UI)
