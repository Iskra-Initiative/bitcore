// -- lower level implementation
// handles direct interaction with the serial port

use serialport::{ClearBuffer, Result};
use serialport::{SerialPort, SerialPortBuilder, SerialPortInfo};
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Duration, Instant};

/// polling read period for the duration of a user-defined timeout
const READ_POLL_SLEEP_MS: u64 = 100;

pub struct SerialConnection {
    port: Box<dyn SerialPort>,
}

impl SerialConnection {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        SerialConnection { port }
    }

    pub fn list() -> io::Result<Vec<SerialPortInfo>> {
        let ports = serialport::available_ports()?;
        Ok(ports)
    }

    pub fn connect(spbuild: SerialPortBuilder) -> io::Result<Self> {
        let mut port = spbuild.open()?;

        // flush to ensure buffer emptiness before writing (TODO: error handling)
        port.flush()?;

        Ok(Self { port })
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

    fn baud_rate(&self) -> Result<u32> {
        self.port.baud_rate()
    }

    fn data_bits(&self) -> Result<serialport::DataBits> {
        self.port.data_bits()
    }

    fn flow_control(&self) -> Result<serialport::FlowControl> {
        self.port.flow_control()
    }

    fn parity(&self) -> Result<serialport::Parity> {
        self.port.parity()
    }

    fn stop_bits(&self) -> Result<serialport::StopBits> {
        self.port.stop_bits()
    }

    fn timeout(&self) -> Duration {
        self.port.timeout()
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()> {
        self.port.set_baud_rate(baud_rate)
    }

    fn set_data_bits(&mut self, data_bits: serialport::DataBits) -> Result<()> {
        self.port.set_data_bits(data_bits)
    }

    fn set_flow_control(&mut self, flow_control: serialport::FlowControl) -> Result<()> {
        self.port.set_flow_control(flow_control)
    }

    fn set_parity(&mut self, parity: serialport::Parity) -> Result<()> {
        self.port.set_parity(parity)
    }

    fn set_stop_bits(&mut self, stop_bits: serialport::StopBits) -> Result<()> {
        self.port.set_stop_bits(stop_bits)
    }

    fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        self.port.set_timeout(timeout)
    }

    fn write_request_to_send(&mut self, _data: bool) -> Result<()> {
        self.port.write_request_to_send(_data)
    }

    fn write_data_terminal_ready(&mut self, _data: bool) -> Result<()> {
        self.port.write_data_terminal_ready(_data)
    }

    fn read_clear_to_send(&mut self) -> Result<bool> {
        self.port.read_clear_to_send()
    }

    fn read_data_set_ready(&mut self) -> Result<bool> {
        self.port.read_data_set_ready()
    }

    fn read_ring_indicator(&mut self) -> Result<bool> {
        self.port.read_ring_indicator()
    }

    fn read_carrier_detect(&mut self) -> Result<bool> {
        self.port.read_carrier_detect()
    }

    fn bytes_to_read(&self) -> Result<u32> {
        self.port.bytes_to_read()
    }

    fn bytes_to_write(&self) -> Result<u32> {
        self.port.bytes_to_write()
    }

    fn clear(&self, buffer_to_clear: ClearBuffer) -> Result<()> {
        self.port.clear(buffer_to_clear)
    }

    fn try_clone(&self) -> Result<Box<dyn SerialPort>> {
        self.port.try_clone()
    }

    fn set_break(&self) -> Result<()> {
        self.port.set_break()
    }

    fn clear_break(&self) -> Result<()> {
        self.port.clear_break()
    }
}

impl Read for SerialConnection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let start_time = Instant::now();

        while start_time.elapsed() < self.timeout() {
            match self.port.bytes_to_read() {
                Ok(bytes) => {
                    if bytes > 0 {
                        match self.port.read(buf) {
                            Ok(bytes_read) => {
                                if bytes_read > 0 {
                                    return Ok(bytes_read);
                                }
                            }

                            Err(e) => {
                                return Err(io::Error::new(
                                    io::ErrorKind::Other,
                                    format!("[core] error reading bytes>: {}", e),
                                ));
                            }
                        };
                    }
                }

                Err(e) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("[core] error reading number of bytes to read>: {}", e),
                    ));
                }
            }

            thread::sleep(Duration::from_millis(READ_POLL_SLEEP_MS));
        }

        // read timeout elapsed
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "Read operation timed out",
        ))
    }
}

impl Write for SerialConnection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.port.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.port.flush()
    }
}
