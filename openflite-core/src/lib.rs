pub mod config;
pub mod device;
pub mod protocol;

#[derive(Debug, Clone)]
pub enum Event {
    DeviceDetected(String),
    SimConnected(String),
    SimDisconnected,
    VariableChanged { name: String, value: f64 },
    CommandSent(String),
}

use crate::device::MobiFlightDevice;
use openflite_connect::SimClient;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct Core {
    event_tx: mpsc::UnboundedSender<Event>,
    devices: Arc<Mutex<Vec<MobiFlightDevice>>>,
    sim_client: Arc<Mutex<Option<Box<dyn SimClient + Send>>>>,
}

impl Core {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Event>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                event_tx: tx,
                devices: Arc::new(Mutex::new(Vec::new())),
                sim_client: Arc::new(Mutex::new(None)),
            },
            rx,
        )
    }

    pub fn set_sim_client(
        &self,
        mut client: Box<dyn SimClient + Send>,
    ) -> Result<(), anyhow::Error> {
        client.connect()?;
        let mut sim = self.sim_client.lock().unwrap();
        *sim = Some(client);
        Ok(())
    }

    pub fn disconnect_sim(&self) {
        let mut sim = self.sim_client.lock().unwrap();
        if let Some(mut client) = sim.take() {
            let _ = client.disconnect();
        }
        self.broadcast(Event::SimDisconnected);
    }

    pub fn scan_devices(&self) -> Result<(), anyhow::Error> {
        let ports = MobiFlightDevice::scan()?;
        let mut devices = self.devices.lock().unwrap();

        for port in ports {
            if !devices.iter().any(|d| d.serial == port) {
                // Using serial as proxy for now
                if let Ok(dev) = MobiFlightDevice::new(&port) {
                    let name = dev.name.clone();
                    devices.push(dev);
                    self.broadcast(Event::DeviceDetected(name));
                }
            }
        }
        Ok(())
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        loop {
            // Check for input events from devices
            {
                let mut devices = self.devices.lock().unwrap();
                for _dev in devices.iter_mut() {
                    // In a real implementation, we would poll the serial port here
                    // for any incoming button/encoder messages.
                }
            }

            // Poll simulator
            {
                let mut sim = self.sim_client.lock().unwrap();
                if let Some(client) = sim.as_mut() {
                    let _ = client.poll();
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    pub fn broadcast(&self, event: Event) {
        let _ = self.event_tx.send(event);
    }

    pub fn get_devices(&self) -> Vec<String> {
        let devices = self.devices.lock().unwrap();
        devices
            .iter()
            .map(|d| format!("{} ({})", d.name, d.board_type))
            .collect()
    }

    pub fn get_all_variables(&self) -> std::collections::HashMap<String, f64> {
        let sim = self.sim_client.lock().unwrap();
        if let Some(client) = sim.as_ref() {
            client.get_all_variables()
        } else {
            std::collections::HashMap::new()
        }
    }
}
