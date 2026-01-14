use anyhow::Result;

pub trait SimClient {
    /// Connect to the simulator
    fn connect(&mut self) -> Result<()>;

    /// Disconnect from the simulator
    fn disconnect(&mut self) -> Result<()>;

    /// Read a variable (Dataref / SimVar)
    fn read_variable(&mut self, variable: &str) -> Result<f64>;

    /// Write to a variable
    fn write_variable(&mut self, variable: &str, value: f64) -> Result<()>;

    /// Poll for new data (non-blocking)
    fn poll(&mut self) -> Result<()>;

    /// Get all currently cached variables
    fn get_all_variables(&self) -> std::collections::HashMap<String, f64>;
}

pub mod dummy;
pub mod msfs;
pub mod xplane;
