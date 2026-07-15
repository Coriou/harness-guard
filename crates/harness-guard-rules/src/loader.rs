//! Compile-time embedded rules (§4: rules ship inside the binary for now)
//! + the type-level citation guarantee (§5.2 point 1).
use crate::schema::{RawRule, Source};

const RULESET_JSON: &str = include_str!("../../../rules/ruleset.json");
const RULE_HISTORY_PERSIST: &str = include_str!("../../../rules/codex/history-persist-01.json");

/// A rule that passed structural validation. The `primary_source` field is
/// non-optional: a rule with any non-`unknown` outcome cannot become a
/// `ValidatedRule` without at least one Source — the type repeats the
/// schema's guarantee.
#[derive(Debug, Clone)]
pub struct ValidatedRule {
    pub raw: RawRule,
    pub primary_source: Source,
}

#[derive(Debug)]
pub struct RuleValidationError(pub String);

impl std::fmt::Display for RuleValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rule validation failed: {}", self.0)
    }
}

impl std::error::Error for RuleValidationError {}

impl ValidatedRule {
    pub fn try_from_raw(raw: RawRule) -> Result<Self, RuleValidationError> {
        let has_cited_outcome = raw.outcomes.iter().any(|o| o.status != "unknown");
        let primary_source = match raw.sources.first() {
            Some(s) if !s.url.is_empty() && !s.retrieved.is_empty() => s.clone(),
            _ if has_cited_outcome => {
                return Err(RuleValidationError(format!(
                    "rule {} has a non-unknown outcome but no usable source",
                    raw.id
                )));
            }
            Some(s) => s.clone(),
            None => {
                return Err(RuleValidationError(format!(
                    "rule {} has no sources at all",
                    raw.id
                )));
            }
        };
        if raw.limitations.is_empty() || raw.unknown_conditions.is_empty() {
            return Err(RuleValidationError(format!(
                "rule {} must declare limitations and unknown_conditions",
                raw.id
            )));
        }
        if raw.observation.allowed_render.is_empty() {
            return Err(RuleValidationError(format!(
                "rule {} has an empty allowed_render allowlist",
                raw.id
            )));
        }
        if raw.tested_versions.is_empty() {
            return Err(RuleValidationError(format!(
                "rule {} has no tested_versions",
                raw.id
            )));
        }
        Ok(ValidatedRule {
            raw,
            primary_source,
        })
    }
}

/// All bundled rules. Panics only on a corrupt embed, which `cargo test`
/// catches before any release build ships.
pub fn load_rules() -> Vec<ValidatedRule> {
    let raw: RawRule = serde_json::from_str(RULE_HISTORY_PERSIST)
        .expect("embedded rule JSON is valid (checked in tests)");
    vec![
        ValidatedRule::try_from_raw(raw)
            .expect("embedded rule passes validation (checked in tests)"),
    ]
}

pub fn ruleset_version() -> String {
    let v: serde_json::Value =
        serde_json::from_str(RULESET_JSON).expect("embedded ruleset.json is valid");
    v["ruleset_version"]
        .as_str()
        .expect("ruleset_version present")
        .to_string()
}
