use crate::SimClient;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:8080";

pub struct MSFSClient {
    connected: bool,
    bridge_url: String,
    client: reqwest::blocking::Client,
    variables: HashMap<String, f64>,
}

impl MSFSClient {
    pub fn new() -> Self {
        Self {
            connected: false,
            bridge_url: DEFAULT_BRIDGE_URL.to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_millis(500))
                .build()
                .unwrap(),
            variables: HashMap::new(),
        }
    }

    pub fn with_url(url: &str) -> Self {
        Self {
            connected: false,
            bridge_url: url.to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_millis(500))
                .build()
                .unwrap(),
            variables: HashMap::new(),
        }
    }
}

impl SimClient for MSFSClient {
    fn connect(&mut self) -> Result<()> {
        // Try to reach the MSFS bridge
        let url = format!("{}/status", self.bridge_url);
        match self.client.get(&url).send() {
            Ok(resp) if resp.status().is_success() => {
                log::info!("Connected to MSFS bridge at {}", self.bridge_url);
                self.connected = true;
                Ok(())
            }
            Ok(resp) => Err(anyhow!("Bridge returned error: {}", resp.status())),
            Err(e) => Err(anyhow!(
                "Failed to connect to MSFS bridge: {}. Is the WASM module installed?",
                e
            )),
        }
    }

    fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        self.variables.clear();
        log::info!("Disconnected from MSFS bridge");
        Ok(())
    }

    fn read_variable(&mut self, variable: &str) -> Result<f64> {
        self.variables
            .get(variable)
            .copied()
            .ok_or_else(|| anyhow!("Variable {} not found", variable))
    }

    fn write_variable(&mut self, variable: &str, value: f64) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected"));
        }

        let url = format!("{}/simvar", self.bridge_url);
        let payload = serde_json::json!({
            "name": variable,
            "value": value
        });

        self.client
            .post(&url)
            .json(&payload)
            .send()
            .map_err(|e| anyhow!("Failed to write variable: {}", e))?;

        Ok(())
    }

    fn execute_command(&mut self, command: &str) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected"));
        }

        let url = format!("{}/command", self.bridge_url);
        let payload = serde_json::json!({
            "event": command
        });

        self.client
            .post(&url)
            .json(&payload)
            .send()
            .map_err(|e| anyhow!("Failed to execute command: {}", e))?;

        log::debug!("Executed MSFS command: {}", command);
        Ok(())
    }

    fn poll(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        let url = format!("{}/simvars", self.bridge_url);
        match self.client.get(&url).send() {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(vars) = resp.json::<HashMap<String, f64>>() {
                    self.variables = vars;
                }
            }
            Ok(_) => {}
            Err(e) => {
                log::warn!("Failed to poll MSFS: {}", e);
            }
        }
        Ok(())
    }

    fn get_all_variables(&self) -> HashMap<String, f64> {
        self.variables.clone()
    }
}
