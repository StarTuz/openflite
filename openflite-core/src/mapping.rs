use crate::config::{Action, MobiFlightProject};
use crate::protocol::Response;
use std::collections::HashMap;

pub struct MappingEngine {
    project: MobiFlightProject,
}

impl MappingEngine {
    pub fn new(project: MobiFlightProject) -> Self {
        Self { project }
    }

    pub fn process_outputs(&self, data: &HashMap<String, f64>) -> Vec<HardwareAction> {
        let mut actions = Vec::new();

        for config in &self.project.outputs.config {
            if !config.active {
                continue;
            }

            let settings = &config.settings;
            if let (Some(source), Some(display)) = (&settings.source, &settings.display) {
                if let Some(&val) = data.get(&source.name) {
                    let mut final_val = val;
                    if let Some(comp) = &settings.comparison {
                        if comp.active {
                            final_val = self.apply_comparison(val, comp);
                        }
                    }

                    actions.push(HardwareAction {
                        serial: display.serial.clone(),
                        pin: display.pin.parse().unwrap_or(0),
                        value: final_val as i32,
                    });
                }
            }
        }

        actions
    }

    pub fn process_inputs(&self, resp: &Response) -> Vec<SimAction> {
        let mut actions = Vec::new();

        if let Response::InputEvent { name, value } = resp {
            // Find input config by name (the hardware pin/device name)
            for config in &self.project.inputs.config {
                if !config.active || config.description != *name {
                    continue;
                }

                if let Some(button) = &config.settings.button {
                    let action = if value == "1" {
                        button.on_press.as_ref()
                    } else {
                        button.on_release.as_ref()
                    };

                    if let Some(action) = action {
                        actions.push(self.create_sim_action(action));
                    }
                }
            }
        }

        actions
    }

    fn create_sim_action(&self, action: &Action) -> SimAction {
        if let Some(cmd) = &action.command {
            SimAction::Command(cmd.clone())
        } else if let Some(dref) = &action.dataref {
            let val = action
                .value
                .as_ref()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);
            SimAction::WriteDataref(dref.clone(), val)
        } else {
            SimAction::None
        }
    }

    fn apply_comparison(&self, val: f64, comp: &crate::config::Comparison) -> f64 {
        let target: f64 = comp.value.parse().unwrap_or(0.0);
        let condition_met = match comp.operand.as_str() {
            ">" => val > target,
            "<" => val < target,
            "==" | "=" => (val - target).abs() < f64::EPSILON,
            ">=" => val >= target,
            "<=" => val <= target,
            "!=" => (val - target).abs() > f64::EPSILON,
            _ => false,
        };

        if condition_met {
            comp.if_value.parse().unwrap_or(1.0)
        } else {
            comp.else_value.parse().unwrap_or(0.0)
        }
    }
}

pub struct HardwareAction {
    pub serial: String,
    pub pin: u8,
    pub value: i32,
}

pub enum SimAction {
    Command(String),
    WriteDataref(String, f64),
    None,
}
