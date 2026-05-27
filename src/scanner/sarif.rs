use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SarifLog {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<Run>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Run {
    pub tool: Tool,
    pub results: Vec<ResultEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tool {
    pub driver: Driver,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Driver {
    pub name: String,
    pub version: String,
    pub rules: Vec<Rule>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub short_description: Message,
    pub help: Message,
    pub properties: RuleProperties,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleProperties {
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultEntry {
    pub rule_id: String,
    pub message: Message,
    pub locations: Vec<Location>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub physical_location: PhysicalLocation,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PhysicalLocation {
    pub artifact_location: ArtifactLocation,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArtifactLocation {
    pub uri: String,
}
