// -- comprehensive tests for bitcore simplified API

use bitcore::{config::RetryConfig, Serial, SerialConfig};
use std::time::Duration;

/// initialize tracing for tests
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

mod unit_tests {
    use super::*;

    #[test]
    fn test_list_ports() {
        init_tracing();

        let result = Serial::list_ports();
        match result {
            Ok(ports) => {
                println!("found {} serial ports", ports.len());
                for (i, port) in ports.iter().enumerate() {
                    println!("  [{}] {:?}", i, port);
                }
                // test should pass regardless of number of ports
            }
            Err(e) => {
                println!("error listing ports: {:?}", e);
                // this might fail on some systems, which is ok for unit tests
            }
        }
    }

    #[test]
    fn test_serial_config() {
        init_tracing();

        // test default config
        let default_config = SerialConfig::default();
        assert_eq!(default_config.baud_rate, 9600);
        assert_eq!(default_config.timeout, Duration::from_secs(1));
        assert_eq!(default_config.retries, 3);

        // test custom config
        let custom_config = SerialConfig::new(115200)
            .timeout(Duration::from_millis(500))
            .retries(5);
        assert_eq!(custom_config.baud_rate, 115200);
        assert_eq!(custom_config.timeout, Duration::from_millis(500));
        assert_eq!(custom_config.retries, 5);
    }

    #[test]
    fn test_connection_to_nonexistent_port() {
        init_tracing();

        // test connection to non-existent port should fail gracefully
        let result = Serial::new("/dev/nonexistent_port_12345");
        assert!(result.is_err());

        // test with custom config should also fail gracefully
        let config = SerialConfig::new(115200);
        let result = Serial::with_config("/dev/nonexistent_port_12345", config);
        assert!(result.is_err());
    }

    #[test]
    fn test_retry_config() {
        init_tracing();

        // test default retry config
        let default_config = RetryConfig::default();
        assert_eq!(default_config.max_attempts, 3);
        assert_eq!(default_config.retry_delay, Duration::from_millis(100));
        assert_eq!(default_config.backoff_multiplier, 1.5);

        // test custom retry config
        let custom_config = RetryConfig::new(5)
            .with_delay(Duration::from_millis(50))
            .with_backoff(2.0);
        assert_eq!(custom_config.max_attempts, 5);
        assert_eq!(custom_config.retry_delay, Duration::from_millis(50));
        assert_eq!(custom_config.backoff_multiplier, 2.0);

        // test delay calculation with exponential backoff
        let delay_0 = custom_config.delay_for_attempt(0);
        let delay_1 = custom_config.delay_for_attempt(1);
        let delay_2 = custom_config.delay_for_attempt(2);

        assert_eq!(delay_0, Duration::from_millis(50));
        assert_eq!(delay_1, Duration::from_millis(100)); // 50 * 2^1
        assert_eq!(delay_2, Duration::from_millis(200)); // 50 * 2^2
    }

    #[test]
    fn test_serial_config_builder_pattern() {
        init_tracing();

        // test builder pattern works correctly
        let config = SerialConfig::new(57600)
            .timeout(Duration::from_millis(250))
            .retries(10);

        assert_eq!(config.baud_rate, 57600);
        assert_eq!(config.timeout, Duration::from_millis(250));
        assert_eq!(config.retries, 10);

        // test that other fields keep defaults
        assert_eq!(config.data_bits, serialport::DataBits::Eight);
        assert_eq!(config.parity, serialport::Parity::None);
        assert_eq!(config.stop_bits, serialport::StopBits::One);
        assert_eq!(config.flow_control, serialport::FlowControl::None);
    }
}
