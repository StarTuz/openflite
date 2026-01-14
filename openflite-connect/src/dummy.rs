use crate::SimClient;
use anyhow::Result;

pub struct DummyClient {
    connected: bool,
    counter: f64,
}

impl DummyClient {
    pub fn new() -> Self {
        Self {
            connected: false,
            counter: 0.0,
        }
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

    fn execute_command(&mut self, command: &str) -> Result<()> {
        log::info!("DummyClient executing command: {}", command);
        Ok(())
    }

    fn poll(&mut self) -> Result<()> {
        if self.connected {
            self.counter += 0.1;
        }
        Ok(())
    }

    fn get_all_variables(&self) -> std::collections::HashMap<String, f64> {
        let mut vars = std::collections::HashMap::new();
        if self.connected {
            vars.insert(
                "sim/flightmodel/position/altitude".to_string(),
                1000.0 + self.counter,
            );
            vars.insert(
                "sim/cockpit2/controls/gear_handle_down".to_string(),
                if self.counter % 20.0 > 10.0 { 1.0 } else { 0.0 },
            );
            vars.insert(
                "sim/flightmodel/engine/ENGN_RPM[0]".to_string(),
                2500.0 + (self.counter.sin() * 100.0),
            );
        }
        vars
    }
}
