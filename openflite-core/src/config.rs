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
    pub config: Vec<Config>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inputs {
    #[serde(rename = "Config", default)]
    pub config: Vec<Config>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    #[serde(rename = "@guid")]
    pub guid: String,
    #[serde(rename = "@active")]
    pub active: bool,
    pub description: String,
    pub settings: ConfigSettings,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigSettings {
    pub source: Option<Source>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Source {
    #[serde(rename = "@type")]
    pub source_type: String,
    #[serde(rename = "@name")]
    pub name: String,
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
