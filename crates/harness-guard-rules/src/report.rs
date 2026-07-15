//! §5.4 sanitized report — simultaneously the --json contract and the ONLY
//! artifact shape. No raw config values ever enter these structs.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct SourceCite {
    pub url: String,
    pub retrieved: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    pub tools_scanned: u32,
    pub warning: u32,
    pub info: u32,
    pub unknown: u32,
    pub stale: u32,
    pub passed: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportValidationError(pub String);

impl std::fmt::Display for ReportValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "report validation failed: {}", self.0)
    }
}

impl std::error::Error for ReportValidationError {}

impl FindingRecord {
    /// Validate the status matrix before a finding is summarized or emitted.
    pub fn validate(&self) -> Result<(), ReportValidationError> {
        if self.rule_id.is_empty() || self.message.is_empty() {
            return self.invalid("rule_id and message must be non-empty");
        }
        if let Some(remediation) = &self.remediation {
            if remediation.summary.is_empty() || remediation.command.is_empty() {
                return self.invalid("remediation summary and command must be non-empty");
            }
        }
        if let Some(source) = &self.source {
            if !valid_https_url(&source.url) || !valid_date(&source.retrieved) {
                return self.invalid("source must contain a valid HTTPS URL and retrieved date");
            }
        }
        if self
            .verify_url
            .as_deref()
            .is_some_and(|url| !valid_https_url(url))
        {
            return self.invalid("verify_url must be a valid HTTPS URL when present");
        }

        match self.status {
            Status::Pass => {
                if self.severity.is_some()
                    || self.confidence.is_none()
                    || self.source.is_none()
                    || self.remediation.is_some()
                    || self.unknown_reason.is_some()
                    || self.stale_reason.is_some()
                {
                    return self.invalid(
                        "pass requires confidence and source and forbids severity, remediation, unknown_reason, and stale_reason",
                    );
                }
            }
            Status::Finding => {
                if self.severity.is_none()
                    || self.confidence.is_none()
                    || self.source.is_none()
                    || self.unknown_reason.is_some()
                    || self.stale_reason.is_some()
                {
                    return self.invalid(
                        "finding requires severity, confidence, and source and forbids unknown_reason and stale_reason",
                    );
                }
            }
            Status::Unknown => {
                if self.severity.is_some()
                    || self.confidence.is_some()
                    || self.source.is_some()
                    || self.remediation.is_some()
                    || self.unknown_reason.as_deref().is_none_or(str::is_empty)
                    || self.stale_reason.is_some()
                {
                    return self.invalid(
                        "unknown requires unknown_reason and forbids severity, confidence, source, remediation, and stale_reason",
                    );
                }
            }
            Status::StaleRuleset => {
                if self.severity.is_some()
                    || self.confidence.is_some()
                    || self.source.is_none()
                    || self.remediation.is_some()
                    || self.unknown_reason.is_some()
                    || self.stale_reason.as_deref().is_none_or(str::is_empty)
                {
                    return self.invalid(
                        "stale-ruleset requires source and stale_reason and forbids severity, confidence, remediation, and unknown_reason",
                    );
                }
            }
        }
        Ok(())
    }

    fn invalid<T>(&self, message: &str) -> Result<T, ReportValidationError> {
        Err(ReportValidationError(format!(
            "finding {} {message}",
            self.rule_id
        )))
    }
}

impl Summary {
    pub fn from_tools(tools: &[ToolReport]) -> Summary {
        Self::try_from_tools(tools)
            .expect("FindingRecord status invariants must hold before summarizing")
    }

    pub fn try_from_tools(tools: &[ToolReport]) -> Result<Summary, ReportValidationError> {
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
                f.validate()?;
                match f.status {
                    Status::Pass => s.passed += 1,
                    Status::Unknown => s.unknown += 1,
                    Status::StaleRuleset => s.stale += 1,
                    Status::Finding => match f
                        .severity
                        .expect("validated finding status always has severity")
                    {
                        Severity::Warning => s.warning += 1,
                        Severity::Info => s.info += 1,
                    },
                }
            }
        }
        Ok(s)
    }
}

fn valid_https_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    !authority.is_empty() && !url.bytes().any(|byte| byte.is_ascii_whitespace())
}

fn valid_date(date: &str) -> bool {
    const DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
        time::macros::format_description!("[year]-[month]-[day]");
    date.len() == 10 && time::Date::parse(date, DATE_FORMAT).is_ok()
}
