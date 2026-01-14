pub mod config;
pub mod device;
pub mod flash;
pub mod mapping;
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
use crate::mapping::MappingEngine;
use crate::protocol::Response;
use openflite_connect::SimClient;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct Core {
    event_tx: mpsc::UnboundedSender<Event>,
    devices: Arc<Mutex<Vec<MobiFlightDevice>>>,
    sim_client: Arc<Mutex<Option<Box<dyn SimClient + Send>>>>,
    mapping_engine: Arc<Mutex<Option<MappingEngine>>>,
    injected_responses: Arc<Mutex<Vec<(String, Response)>>>,
}

impl Core {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Event>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                event_tx: tx,
                devices: Arc::new(Mutex::new(Vec::new())),
                sim_client: Arc::new(Mutex::new(None)),
                mapping_engine: Arc::new(Mutex::new(None)),
                injected_responses: Arc::new(Mutex::new(Vec::new())),
            },
            rx,
        )
    }

    pub fn load_config(&self, xml_content: &str) -> Result<(), anyhow::Error> {
        let project = crate::config::MobiFlightProject::load(xml_content)?;
        let mut engine = self.mapping_engine.lock().unwrap();
        *engine = Some(MappingEngine::new(project));
        Ok(())
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
            let hardware_responses = self.collect_hardware_events();
            let hardware_actions = self.process_simulation_sync(hardware_responses);
            self.apply_hardware_outputs(hardware_actions);

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    fn collect_hardware_events(&self) -> Vec<(String, Response)> {
        let mut hardware_responses = Vec::new();
        // 1. Process injected responses first
        {
            let mut injected = self.injected_responses.lock().unwrap();
            hardware_responses.append(&mut *injected);
        }

        // 2. Poll physical devices
        let mut devices = self.devices.lock().unwrap();
        for dev in devices.iter_mut() {
            let resps = dev.poll_events();
            for resp in resps {
                hardware_responses.push((dev.name.clone(), resp));
            }
        }
        hardware_responses
    }

    fn process_simulation_sync(
        &self,
        hardware_responses: Vec<(String, Response)>,
    ) -> Vec<crate::mapping::HardwareAction> {
        let mut hardware_actions = Vec::new();
        let mut sim = self.sim_client.lock().unwrap();

        if let Some(client) = sim.as_mut() {
            let _ = client.poll();

            let mapping = self.mapping_engine.lock().unwrap();
            if let Some(engine) = mapping.as_ref() {
                // A. Sim -> Hardware
                let data = client.get_all_variables();
                hardware_actions = engine.process_outputs(&data);

                // B. Hardware -> Sim
                for (dev_name, resp) in hardware_responses {
                    // Update UI cache for inputs too
                    if let Response::InputEvent {
                        name: pin_name,
                        value,
                    } = &resp
                    {
                        self.broadcast(Event::VariableChanged {
                            name: format!("{}:{}", dev_name, pin_name),
                            value: value.parse().unwrap_or(0.0),
                        });
                    }

                    let sim_actions = engine.process_inputs(&resp);
                    for action in sim_actions {
                        match action {
                            crate::mapping::SimAction::Command(cmd) => {
                                let _ = client.execute_command(&cmd);
                            }
                            crate::mapping::SimAction::WriteDataref(dref, val) => {
                                let _ = client.write_variable(&dref, val);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        hardware_actions
    }

    fn apply_hardware_outputs(&self, hardware_actions: Vec<crate::mapping::HardwareAction>) {
        if !hardware_actions.is_empty() {
            let mut devices = self.devices.lock().unwrap();
            for action in hardware_actions {
                match action {
                    crate::mapping::HardwareAction::SetPin { serial, pin, value } => {
                        if let Some(dev) = devices.iter_mut().find(|d| d.serial == serial) {
                            let _ = dev.set_pin(pin, value);
                        }
                    }
                    crate::mapping::HardwareAction::Set7Segment {
                        serial,
                        module,
                        index,
                        value,
                    } => {
                        if let Some(dev) = devices.iter_mut().find(|d| d.serial == serial) {
                            let _ = dev.set_7segment(module, index, &value);
                        }
                    }
                    crate::mapping::HardwareAction::SetLCD {
                        serial,
                        display_id,
                        line,
                        text,
                    } => {
                        if let Some(dev) = devices.iter_mut().find(|d| d.serial == serial) {
                            let _ = dev.set_lcd(display_id, line, &text);
                        }
                    }
                }
            }
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

    pub fn inject_hardware_response(&self, dev_name: &str, resp: Response) {
        let mut injected = self.injected_responses.lock().unwrap();
        injected.push((dev_name.to_string(), resp));
    }
}
