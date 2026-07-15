//! Serde mirror of schemas/rule.schema.json and schemas/source.schema.json.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawRule {
    pub schema_version: String,
    pub id: String,
    pub tool: String,
    pub category: String,
    pub title: String,
    pub why_it_matters: String,
    pub os: Vec<String>,
    pub scopes: Vec<String>,
    pub auth_prerequisites: Option<String>,
    pub observation: Observation,
    pub outcomes: Vec<RawOutcome>,
    pub tested_versions: Vec<TestedVersion>,
    pub sources: Vec<Source>,
    pub limitations: Vec<String>,
    pub unknown_conditions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub file: String,
    pub key: String,
    #[serde(rename = "type")]
    pub value_type: String,
    pub allowed_render: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawOutcome {
    pub when: String,
    pub status: String, // "pass" | "finding" | "unknown" (schema-constrained)
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    pub message: String,
    #[serde(default)]
    pub remediation: Option<Remediation>,
    #[serde(default)]
    pub unknown_reason: Option<String>,
    #[serde(default)]
    pub verify_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Remediation {
    pub summary: String,
    pub command: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub schema_version: String,
    pub url: String,
    pub publisher: String,
    pub title: String,
    pub evidence_class: String,
    pub retrieved: String,
    pub content_hash: String,
    pub archived_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestedVersion {
    pub min: String, // may carry the MDN "<=" prefix
    pub max: String,
    pub verified_on: String,
}
