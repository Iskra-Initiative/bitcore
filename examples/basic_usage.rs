// -- basic usage example for bitcore

use bitcore::{Serial, SerialConfig};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("bitcore serial communication example");
    println!("====================================");

    // list available ports
    println!("\navailable serial ports:");
    match Serial::list_ports() {
        Ok(ports) => {
            if ports.is_empty() {
                println!("  no serial ports found");
            } else {
                for (i, port) in ports.iter().enumerate() {
                    println!("  [{}] {} - {:?}", i, port.port_name, port.port_type);
                }
            }
        }
        Err(e) => {
            eprintln!("error listing ports: {}", e);
        }
    }

    println!("\ndemonstrating simplified API:");

    // example 1: basic usage with defaults
    println!("\n1. basic usage (would fail without hardware):");
    println!("   let serial = Serial::new(\"/dev/ttyUSB0\")?;");
    println!("   serial.write_str(\"hello world\")?;");
    println!("   let response = serial.read_line()?;");

    // example 2: custom configuration
    println!("\n2. custom configuration:");
    let config = SerialConfig::new(115200)
        .timeout(Duration::from_millis(500))
        .retries(5);
    println!("   let config = SerialConfig::new(115200)");
    println!("       .timeout(Duration::from_millis(500))");
    println!("       .retries(5);");
    println!("   let serial = Serial::with_config(\"/dev/ttyUSB0\", config)?;");
    println!(
        "   ✓ configured: {} baud, {}ms timeout, {} retries",
        config.baud_rate,
        config.timeout.as_millis(),
        config.retries
    );

    // example 3: demonstrate error handling
    println!("\n3. error handling:");
    match Serial::new("/dev/nonexistent") {
        Ok(_) => println!("   unexpected success"),
        Err(e) => println!("   ✓ correctly failed to connect: {}", e),
    }

    // example 4: show the old vs new API comparison
    println!("\n4. API comparison:");
    println!("   OLD (verbose):");
    println!("     let connection: SharedConnection = Arc::new(Mutex::new(None));");
    println!("     let port_builder = serialport::new(\"/dev/ttyUSB0\", 9600);");
    println!("     connect(&connection, port_builder)?;");
    println!("     write(&connection, data, 3)?;");
    println!("     disconnect(&connection)?;");
    println!();
    println!("   NEW (simple):");
    println!("     let serial = Serial::new(\"/dev/ttyUSB0\")?;");
    println!("     serial.write(data)?;");
    println!("     // automatic cleanup on drop");

    println!("\n5. convenience methods:");
    println!("   serial.write_str(\"AT\\r\\n\")?;           // write string");
    println!("   let response = serial.read_line()?;      // read until newline");
    println!("   serial.read_exact(&mut buffer)?;        // read exact bytes");
    println!("   let connected = serial.is_connected();   // check status");

    println!("\nexample completed successfully!");
    println!("the new API is much simpler and more user-friendly!");

    Ok(())
}
