// -- lower level implementation
// handles direct interaction with the serial port

use serialport::{ClearBuffer, SerialPort, SerialPortBuilder, SerialPortInfo};
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, trace, warn};

/// default polling interval for read operations (optimized from 100ms to 10ms)
const DEFAULT_POLL_INTERVAL_MS: u64 = 10;

pub struct SerialConnection {
    port: Box<dyn SerialPort>,
    poll_interval: Duration,
}

impl SerialConnection {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        SerialConnection {
            port,
            poll_interval: Duration::from_millis(DEFAULT_POLL_INTERVAL_MS),
        }
    }

    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub fn list() -> io::Result<Vec<SerialPortInfo>> {
        let ports = serialport::available_ports()?;
        Ok(ports)
    }

    pub fn connect(spbuild: SerialPortBuilder) -> io::Result<Self> {
        let mut port = spbuild.open()?;

        // flush to ensure buffer emptiness before writing
        port.flush()?;

        Ok(Self {
            port,
            poll_interval: Duration::from_millis(DEFAULT_POLL_INTERVAL_MS),
        })
    }

    pub fn disconnect(self) -> io::Result<()> {
        drop(self.port);
        Ok(())
    }
}

/// serial port driver implementation
impl SerialPort for SerialConnection {
    fn name(&self) -> Option<String> {
        self.port.name()
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        self.port.baud_rate()
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        self.port.data_bits()
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        self.port.flow_control()
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        self.port.parity()
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        self.port.stop_bits()
    }

    fn timeout(&self) -> Duration {
        self.port.timeout()
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
        self.port.set_baud_rate(baud_rate)
    }

    fn set_data_bits(&mut self, data_bits: serialport::DataBits) -> serialport::Result<()> {
        self.port.set_data_bits(data_bits)
    }

    fn set_flow_control(
        &mut self,
        flow_control: serialport::FlowControl,
    ) -> serialport::Result<()> {
        self.port.set_flow_control(flow_control)
    }

    fn set_parity(&mut self, parity: serialport::Parity) -> serialport::Result<()> {
        self.port.set_parity(parity)
    }

    fn set_stop_bits(&mut self, stop_bits: serialport::StopBits) -> serialport::Result<()> {
        self.port.set_stop_bits(stop_bits)
    }

    fn set_timeout(&mut self, timeout: Duration) -> serialport::Result<()> {
        self.port.set_timeout(timeout)
    }

    fn write_request_to_send(&mut self, data: bool) -> serialport::Result<()> {
        self.port.write_request_to_send(data)
    }

    fn write_data_terminal_ready(&mut self, data: bool) -> serialport::Result<()> {
        self.port.write_data_terminal_ready(data)
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        self.port.read_clear_to_send()
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        self.port.read_data_set_ready()
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        self.port.read_ring_indicator()
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        self.port.read_carrier_detect()
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        self.port.bytes_to_read()
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        self.port.bytes_to_write()
    }

    fn clear(&self, buffer_to_clear: ClearBuffer) -> serialport::Result<()> {
        self.port.clear(buffer_to_clear)
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        self.port.try_clone()
    }

    fn set_break(&self) -> serialport::Result<()> {
        self.port.set_break()
    }

    fn clear_break(&self) -> serialport::Result<()> {
        self.port.clear_break()
    }
}

impl Read for SerialConnection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let start_time = Instant::now();
        let timeout = self.timeout();

        trace!("starting read operation with timeout {:?}", timeout);

        while start_time.elapsed() < timeout {
            match self.port.bytes_to_read() {
                Ok(bytes) => {
                    if bytes > 0 {
                        trace!("found {} bytes available to read", bytes);
                        match self.port.read(buf) {
                            Ok(bytes_read) => {
                                if bytes_read > 0 {
                                    debug!("successfully read {} bytes", bytes_read);
                                    return Ok(bytes_read);
                                }
                            }
                            Err(e) => {
                                error!("error reading bytes: {}", e);
                                return Err(io::Error::other(format!("error reading bytes: {e}")));
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("error checking bytes to read: {}", e);
                    return Err(io::Error::other(format!(
                        "error checking bytes to read: {e}"
                    )));
                }
            }

            // optimized polling interval
            thread::sleep(self.poll_interval);
        }

        // read timeout elapsed
        warn!("read operation timed out after {:?}", timeout);
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "read operation timed out",
        ))
    }
}

impl Write for SerialConnection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        trace!("writing {} bytes", buf.len());
        match self.port.write(buf) {
            Ok(bytes_written) => {
                debug!("successfully wrote {} bytes", bytes_written);
                Ok(bytes_written)
            }
            Err(e) => {
                error!("error writing bytes: {}", e);
                Err(e)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        trace!("flushing serial port");
        match self.port.flush() {
            Ok(()) => {
                debug!("successfully flushed serial port");
                Ok(())
            }
            Err(e) => {
                error!("error flushing serial port: {}", e);
                Err(e)
            }
        }
    }
}
