//! Canonical `SystemSpec` data model.
//!
//! Strongly-typed, serializable representation of a system specification
//! synthesized from structured natural language narratives or input documents.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecMetadata {
    pub title: String,
    pub description: String,
    pub source_type: String,
    pub source_files: Vec<String>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub wit_file: String,
    #[serde(default)]
    pub world: String,
    #[serde(default)]
    pub export_interfaces: Vec<String>,
    #[serde(default)]
    pub import_interfaces: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequirementSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(rename = "ref", default)]
    pub ref_target: String,
}

impl Default for RequirementSource {
    fn default() -> Self {
        Self {
            source_type: "narrative".to_string(),
            ref_target: String::new(),
        }
    }
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequirementSpec {
    pub id: String,
    pub component: String,
    pub title: String,
    pub statement: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub source: RequirementSource,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
}

fn default_status() -> String {
    "proposed".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntitySpec {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub fields: Vec<EntityField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationSpec {
    pub name: String,
    pub component: String,
    pub wit_interface: String,
    #[serde(default)]
    pub params: Vec<OperationParam>,
    pub returns: String,
    #[serde(default)]
    pub rules_applied: Vec<String>,
    #[serde(default)]
    pub effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemSpec {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none", default)]
    pub schema: Option<String>,
    pub spec_version: String,
    pub metadata: SpecMetadata,
    #[serde(default)]
    pub components: Vec<ComponentSpec>,
    #[serde(default)]
    pub requirements: Vec<RequirementSpec>,
    #[serde(default)]
    pub entities: Vec<EntitySpec>,
    #[serde(default)]
    pub operations: Vec<OperationSpec>,
}

impl SystemSpec {
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    #[must_use]
    pub fn find_component(&self, id: &str) -> Option<&ComponentSpec> {
        self.components.iter().find(|c| c.id == id)
    }

    #[must_use]
    pub fn find_requirement(&self, id: &str) -> Option<&RequirementSpec> {
        self.requirements.iter().find(|r| r.id == id)
    }
}
