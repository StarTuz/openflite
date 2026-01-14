use crate::protocol::{Command, Response};
use anyhow::{anyhow, Result};
use serialport::SerialPort;
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

pub struct MobiFlightDevice {
    port: Box<dyn SerialPort>,
    pub name: String,
    pub board_type: String,
    pub serial: String,
    pub version: String,
}

impl MobiFlightDevice {
    pub fn new(port_name: &str) -> Result<Self> {
        let port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(500))
            .open()?;

        let mut dev = Self {
            port,
            name: "Unknown".to_string(),
            board_type: "Unknown".to_string(),
            serial: "Unknown".to_string(),
            version: "Unknown".to_string(),
        };

        dev.update_info()?;

        Ok(dev)
    }

    pub fn update_info(&mut self) -> Result<()> {
        self.send_command(Command::GetInfo)?;

        let mut reader = BufReader::new(&mut self.port);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        if let Some(Response::Info {
            name,
            board_type,
            serial,
            version,
        }) = Response::parse(&line)
        {
            self.name = name;
            self.board_type = board_type;
            self.serial = serial;
            self.version = version;
            Ok(())
        } else {
            Err(anyhow!("Failed to parse info response: {}", line))
        }
    }

    pub fn send_command(&mut self, cmd: Command) -> Result<()> {
        let serialized = cmd.serialize();
        self.port.write_all(serialized.as_bytes())?;
        self.port.flush()?;
        Ok(())
    }

    pub fn scan() -> Result<Vec<String>> {
        let ports = serialport::available_ports()?;
        Ok(ports.into_iter().map(|p| p.port_name).collect())
    }
}
