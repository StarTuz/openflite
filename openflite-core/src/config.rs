use anyhow::Result;
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MobiFlightProject {
    pub outputs: Outputs,
    pub inputs: Inputs,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Outputs {
    #[serde(rename = "Config", default)]
    pub config: Vec<OutputConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inputs {
    #[serde(rename = "Config", default)]
    pub config: Vec<InputConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OutputConfig {
    #[serde(rename = "@guid")]
    pub guid: String,
    #[serde(rename = "@active")]
    pub active: bool,
    pub description: String,
    pub settings: ConfigSettings,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InputConfig {
    #[serde(rename = "@guid")]
    pub guid: String,
    #[serde(rename = "@active")]
    pub active: bool,
    pub description: String,
    pub settings: InputSettings,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InputSettings {
    pub button: Option<ButtonAction>,
    pub encoder: Option<EncoderAction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ButtonAction {
    pub on_press: Option<Action>,
    pub on_release: Option<Action>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EncoderAction {
    pub on_left: Option<Action>,
    pub on_right: Option<Action>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Action {
    #[serde(rename = "@type")]
    pub action_type: String, // e.g., "XplaneAction"
    #[serde(rename = "@cmd")]
    pub command: Option<String>,
    #[serde(rename = "@dataref")]
    pub dataref: Option<String>,
    #[serde(rename = "@value")]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigSettings {
    pub source: Option<Source>,
    pub comparison: Option<Comparison>,
    pub display: Option<Display>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Source {
    #[serde(rename = "@type")]
    pub source_type: String,
    #[serde(rename = "@name")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Comparison {
    #[serde(rename = "@active")]
    pub active: bool,
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(rename = "@operand")]
    pub operand: String,
    #[serde(rename = "@ifValue")]
    pub if_value: String,
    #[serde(rename = "@elseValue")]
    pub else_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Display {
    #[serde(rename = "@type")]
    pub display_type: String,
    #[serde(rename = "@serial")]
    pub serial: String,
    #[serde(rename = "@trigger")]
    pub trigger: String,
    #[serde(rename = "@pin")]
    pub pin: String,
}

impl MobiFlightProject {
    pub fn load(xml_content: &str) -> Result<Self> {
        let project: MobiFlightProject = from_str(xml_content)?;
        Ok(project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_xml() {
        let xml = r#"
            <MobiFlightProject>
                <Outputs>
                    <Config guid="123" active="true">
                        <Description>Test Output</Description>
                        <Settings>
                            <Source type="SimConnect" name="L:TestVar" />
                        </Settings>
                    </Config>
                </Outputs>
                <Inputs>
                    <Config guid="456" active="false">
                        <Description>Test Input</Description>
                        <Settings>
                        </Settings>
                    </Config>
                </Inputs>
            </MobiFlightProject>
        "#;
        let project = MobiFlightProject::load(xml).unwrap();
        assert_eq!(project.outputs.config.len(), 1);
        assert_eq!(project.outputs.config[0].description, "Test Output");
    }
}
