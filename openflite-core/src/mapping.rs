use crate::config::MobiFlightProject;
use std::collections::HashMap;

pub struct MappingEngine {
    project: MobiFlightProject,
}

impl MappingEngine {
    pub fn new(project: MobiFlightProject) -> Self {
        Self { project }
    }

    pub fn process_outputs(&self, data: &HashMap<String, f64>) -> Vec<MappingAction> {
        let mut actions = Vec::new();

        for config in &self.project.outputs.config {
            if !config.active {
                continue;
            }

            let settings = &config.settings;
            if let (Some(source), Some(display)) = (&settings.source, &settings.display) {
                if let Some(&val) = data.get(&source.name) {
                    // Basic comparison logic
                    let mut final_val = val;
                    if let Some(comp) = &settings.comparison {
                        if comp.active {
                            final_val = self.apply_comparison(val, comp);
                        }
                    }

                    actions.push(MappingAction {
                        serial: display.serial.clone(),
                        pin: display.pin.parse().unwrap_or(0),
                        value: final_val as i32,
                    });
                }
            }
        }

        actions
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

pub struct MappingAction {
    pub serial: String,
    pub pin: u8,
    pub value: i32,
}
