use crate::SimClient;
use anyhow::Result;

pub struct DummyClient {
    connected: bool,
}

impl DummyClient {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl SimClient for DummyClient {
    fn connect(&mut self) -> Result<()> {
        self.connected = true;
        log::info!("DummyClient connected");
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        log::info!("DummyClient disconnected");
        Ok(())
    }

    fn read_variable(&mut self, _variable: &str) -> Result<f64> {
        Ok(0.0)
    }

    fn write_variable(&mut self, _variable: &str, _value: f64) -> Result<()> {
        Ok(())
    }

    fn poll(&mut self) -> Result<()> {
        Ok(())
    }
}
