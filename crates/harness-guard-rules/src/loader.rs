//! Compile-time embedded rules (§4: rules ship inside the binary for now)
//! + the type-level citation guarantee (§5.2 point 1).
use crate::schema::{RawRule, Source};

const RULESET_JSON: &str = include_str!("../../../rules/ruleset.json");
const RULE_HISTORY_PERSIST: &str = include_str!("../../../rules/codex/history-persist-01.json");

/// A rule that passed structural validation. The `primary_source` field is
/// non-optional: only a rule with a validated Source can become a
/// `ValidatedRule` — the type strengthens the schema's citation guarantee.
#[derive(Debug, Clone)]
pub struct ValidatedRule {
    raw: RawRule,
    primary_source: Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleValidationError(pub String);

impl std::fmt::Display for RuleValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rule validation failed: {}", self.0)
    }
}

impl std::error::Error for RuleValidationError {}

impl ValidatedRule {
    pub fn try_from_raw(raw: RawRule) -> Result<Self, RuleValidationError> {
        validate_rule(&raw)?;

        let primary_source = raw.sources.first().cloned().ok_or_else(|| {
            RuleValidationError(format!("rule {} has no validated source", raw.id))
        })?;
        Ok(ValidatedRule {
            raw,
            primary_source,
        })
    }

    /// The schema-mirrored rule after all runtime invariants have passed.
    pub fn raw(&self) -> &RawRule {
        &self.raw
    }

    /// The validated citation used for non-unknown report outcomes.
    pub fn primary_source(&self) -> &Source {
        &self.primary_source
    }
}

fn validate_rule(raw: &RawRule) -> Result<(), RuleValidationError> {
    if raw.schema_version != "1.0" {
        return invalid(raw, "schema_version must be 1.0");
    }
    if raw.id.is_empty()
        || !raw.id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
    {
        return invalid(raw, "id must be non-empty lowercase kebab-case");
    }
    if raw.tool != "codex" {
        return invalid(raw, "tool must be codex in this slice");
    }
    if raw.title.is_empty() || raw.why_it_matters.is_empty() {
        return invalid(raw, "title and why_it_matters must be non-empty");
    }
    if raw.observation.file.is_empty()
        || raw.observation.key.is_empty()
        || !matches!(raw.observation.value_type.as_str(), "enum" | "bool")
        || raw.observation.allowed_render.is_empty()
        || raw.observation.allowed_render.iter().any(String::is_empty)
    {
        return invalid(
            raw,
            "observation is malformed or has an empty allowed_render allowlist",
        );
    }
    if raw.outcomes.is_empty() {
        return invalid(raw, "outcomes must not be empty");
    }
    for outcome in &raw.outcomes {
        validate_outcome(raw, outcome)?;
    }
    if raw.sources.is_empty() {
        return invalid(raw, "at least one validated source is required");
    }
    for source in &raw.sources {
        validate_source(raw, source)?;
    }
    if raw.tested_versions.is_empty() {
        return invalid(raw, "tested_versions must not be empty");
    }
    for tested in &raw.tested_versions {
        validate_tested_version(raw, tested)?;
    }
    if raw.limitations.is_empty()
        || raw.limitations.iter().any(String::is_empty)
        || raw.unknown_conditions.is_empty()
        || raw.unknown_conditions.iter().any(String::is_empty)
    {
        return invalid(
            raw,
            "limitations and unknown_conditions must contain non-empty entries",
        );
    }
    Ok(())
}

fn validate_outcome(
    raw: &RawRule,
    outcome: &crate::schema::RawOutcome,
) -> Result<(), RuleValidationError> {
    if outcome.when.is_empty() || outcome.message.is_empty() {
        return invalid(raw, "outcome when and message must be non-empty");
    }
    if let Some(remediation) = &outcome.remediation {
        if remediation.summary.is_empty() || remediation.command.is_empty() {
            return invalid(raw, "remediation summary and command must be non-empty");
        }
    }
    if let Some(verify_url) = &outcome.verify_url {
        if !valid_https_url(verify_url) {
            return invalid(raw, "outcome verify_url must be a valid HTTPS URL");
        }
    }

    match outcome.status.as_str() {
        "pass" => {
            if outcome.severity.is_some()
                || !valid_confidence(outcome.confidence.as_deref())
                || outcome.remediation.is_some()
                || outcome.unknown_reason.is_some()
                || outcome.verify_url.is_some()
            {
                return invalid(
                    raw,
                    "pass outcome requires non-null confidence and forbids severity, remediation, unknown_reason, and verify_url",
                );
            }
        }
        "finding" => {
            if !matches!(outcome.severity.as_deref(), Some("info" | "warning"))
                || !valid_confidence(outcome.confidence.as_deref())
                || outcome.unknown_reason.is_some()
                || outcome.verify_url.is_some()
            {
                return invalid(
                    raw,
                    "finding outcome requires severity and confidence and forbids unknown-only fields",
                );
            }
        }
        "unknown" => {
            if outcome.severity.is_some()
                || outcome.confidence.is_some()
                || outcome.remediation.is_some()
                || outcome.unknown_reason.as_deref().is_none_or(str::is_empty)
            {
                return invalid(
                    raw,
                    "unknown outcome requires unknown_reason and forbids severity, confidence, and remediation",
                );
            }
        }
        other => {
            return invalid(raw, &format!("unknown outcome status {other:?}"));
        }
    }
    Ok(())
}

fn validate_source(raw: &RawRule, source: &Source) -> Result<(), RuleValidationError> {
    if source.schema_version != "1.0" {
        return invalid(raw, "source schema_version must be 1.0");
    }
    if !valid_https_url(&source.url) {
        return invalid(raw, "source url must be a valid HTTPS URL");
    }
    if source.publisher.is_empty() || source.title.is_empty() {
        return invalid(raw, "source publisher and title must be non-empty");
    }
    if !matches!(
        source.evidence_class.as_str(),
        "local-observation"
            | "official-documentation"
            | "official-policy"
            | "independent-reproduction"
            | "inference"
    ) {
        return invalid(raw, "source evidence_class is not recognized");
    }
    if !valid_date(&source.retrieved) {
        return invalid(raw, "source retrieved must be a valid YYYY-MM-DD date");
    }
    if !valid_content_hash(&source.content_hash) {
        return invalid(
            raw,
            "source content_hash must be sha256 plus 64 lowercase hex digits",
        );
    }
    if source
        .archived_url
        .as_deref()
        .is_some_and(|url| !valid_https_url(url))
    {
        return invalid(
            raw,
            "source archived_url must be a valid HTTPS URL when present",
        );
    }
    Ok(())
}

fn validate_tested_version(
    raw: &RawRule,
    tested: &crate::schema::TestedVersion,
) -> Result<(), RuleValidationError> {
    let maximum = parse_version_triplet(&tested.max)
        .ok_or_else(|| RuleValidationError(format!("rule {} has malformed max version", raw.id)))?;
    let (unbounded_below, minimum_text) = match tested.min.strip_prefix("<=") {
        Some(version) => (true, version),
        None => (false, tested.min.as_str()),
    };
    let minimum = parse_version_triplet(minimum_text)
        .ok_or_else(|| RuleValidationError(format!("rule {} has malformed min version", raw.id)))?;
    if (unbounded_below && minimum != maximum) || (!unbounded_below && minimum > maximum) {
        return invalid(raw, "tested version bounds are inconsistent");
    }
    if !valid_date(&tested.verified_on) {
        return invalid(
            raw,
            "tested version verified_on must be a valid YYYY-MM-DD date",
        );
    }
    Ok(())
}

fn valid_confidence(confidence: Option<&str>) -> bool {
    matches!(confidence, Some("low" | "medium" | "high"))
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

fn valid_content_hash(hash: &str) -> bool {
    hash.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    })
}

fn parse_version_triplet(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let major = parse_version_part(parts.next()?)?;
    let minor = parse_version_part(parts.next()?)?;
    let patch = parse_version_part(parts.next()?)?;
    parts.next().is_none().then_some((major, minor, patch))
}

fn parse_version_part(part: &str) -> Option<u64> {
    (!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| part.parse().ok())
        .flatten()
}

fn invalid<T>(raw: &RawRule, message: &str) -> Result<T, RuleValidationError> {
    Err(RuleValidationError(format!("rule {} {message}", raw.id)))
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
