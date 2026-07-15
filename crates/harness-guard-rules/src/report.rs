//! §5.4 sanitized report — simultaneously the --json contract and the ONLY
//! artifact shape. No raw config values ever enter these structs.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: String,
    pub harness_guard_version: String,
    pub ruleset_version: String,
    pub scanned_at: String,
    pub network_requests_made: u32, // always 0
    pub platform: Platform,
    pub tools: Vec<ToolReport>,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolReport {
    pub tool: String,
    pub detected_version: Option<String>,
    pub config_paths: Vec<String>,
    pub detection_confidence: Confidence,
    pub rules_last_verified_version: Option<String>,
    pub rules_verified_date: Option<String>,
    pub version_in_range: bool,
    pub findings: Vec<FindingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingRecord {
    pub rule_id: String,
    pub status: Status,
    pub severity: Option<Severity>,
    pub confidence: Option<Confidence>,
    pub evidence_class: Option<String>,
    pub message: String,
    pub observation: Option<String>, // allowlisted rendering ONLY, or None
    pub remediation: Option<crate::schema::Remediation>,
    pub source: Option<SourceCite>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub limitations: Vec<String>,
    pub unknown_reason: Option<String>,
    pub verify_url: Option<String>,
    pub stale_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Pass,
    Finding,
    Unknown,
    StaleRuleset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
} // Ord: Info < Warning (fail-on threshold relies on this)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCite {
    pub url: String,
    pub retrieved: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub tools_scanned: u32,
    pub warning: u32,
    pub info: u32,
    pub unknown: u32,
    pub stale: u32,
    pub passed: u32,
}

impl Summary {
    pub fn from_tools(tools: &[ToolReport]) -> Summary {
        let mut s = Summary {
            tools_scanned: tools.len() as u32,
            warning: 0,
            info: 0,
            unknown: 0,
            stale: 0,
            passed: 0,
        };
        for t in tools {
            for f in &t.findings {
                match f.status {
                    Status::Pass => s.passed += 1,
                    Status::Unknown => s.unknown += 1,
                    Status::StaleRuleset => s.stale += 1,
                    Status::Finding => match f.severity {
                        Some(Severity::Warning) => s.warning += 1,
                        _ => s.info += 1,
                    },
                }
            }
        }
        s
    }
}
