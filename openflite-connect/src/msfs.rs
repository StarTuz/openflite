use crate::SimClient;
use anyhow::{anyhow, Result};

pub struct MSFSClient {
    connected: bool,
}

impl MSFSClient {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl SimClient for MSFSClient {
    fn connect(&mut self) -> Result<()> {
        // In the future, use simconnect-sdk
        // Note: MSFS on Linux requires networked SimConnect (SimConnect.xml)
        log::info!("MSFSClient connection initialized (skeleton)");
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn read_variable(&mut self, _variable: &str) -> Result<f64> {
        Err(anyhow!("MSFS read not yet implemented"))
    }

    fn write_variable(&mut self, _variable: &str, _value: f64) -> Result<()> {
        Err(anyhow!("MSFS write not yet implemented"))
    }

    fn poll(&mut self) -> Result<()> {
        Ok(())
    }
}
