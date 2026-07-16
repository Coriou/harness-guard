//! Compile-time embedded rules (§4: rules ship inside the binary for now)
//! + the type-level citation guarantee (§5.2 point 1).
use crate::schema::{MatchSpec, MatchValue, Observation, RawRule, Source};

const RULESET_JSON: &str = include_str!("../../../rules/ruleset.json");

include!(concat!(env!("OUT_DIR"), "/embedded_rules.rs"));

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
    if raw.schema_version != "1.1" {
        return invalid(raw, "schema_version must be 1.1");
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
    if !matches!(raw.tool.as_str(), "codex" | "claude-code" | "grok-build") {
        return invalid(raw, "tool must be codex, claude-code, or grok-build");
    }
    if !matches!(
        raw.category.as_str(),
        "retention"
            | "telemetry"
            | "training"
            | "transfer"
            | "sync"
            | "permissions"
            | "sandbox"
            | "network"
    ) {
        return invalid(raw, "category is not recognized");
    }
    if raw.os.is_empty()
        || raw
            .os
            .iter()
            .any(|os| !matches!(os.as_str(), "macos" | "linux" | "windows"))
    {
        return invalid(raw, "os must contain only macos, linux, or windows");
    }
    if raw.scopes.is_empty()
        || raw
            .scopes
            .iter()
            .any(|scope| !matches!(scope.as_str(), "user" | "project" | "local" | "managed"))
    {
        return invalid(
            raw,
            "scopes must contain only user, project, local, or managed",
        );
    }
    if raw.title.is_empty() || raw.why_it_matters.is_empty() {
        return invalid(raw, "title and why_it_matters must be non-empty");
    }
    if raw.unknown_subject.is_empty() {
        return invalid(raw, "unknown_subject must be non-empty");
    }
    if raw.observation.file.is_empty()
        || raw.observation.key.is_empty()
        || !matches!(
            raw.observation.value_type.as_str(),
            "enum" | "bool" | "integer"
        )
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
    validate_match_semantics(raw)?;
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

/// §6.3: proves totality of a rule's declarative `match` outcomes so the
/// engine (Task 6) can assume every extracted value maps to exactly one
/// outcome, deterministically, with no fallthrough.
fn validate_match_semantics(raw: &RawRule) -> Result<(), RuleValidationError> {
    let observation = &raw.observation;
    let value_type = observation.value_type.as_str();

    // Integer observations render from the parsed i64, never a string
    // allowlist (§5.7); pin allowed_render to exactly ["unset"].
    if value_type == "integer" {
        if observation.integer_bounds.is_none() {
            return invalid(raw, "integer observations require integer_bounds");
        }
        if observation.allowed_render != ["unset".to_string()] {
            return invalid(
                raw,
                "integer observations must set allowed_render to [\"unset\"]",
            );
        }
        let bounds = observation.integer_bounds.expect("checked above");
        if bounds.min > bounds.max {
            return invalid(raw, "integer_bounds min must be <= max");
        }
    } else {
        if observation.integer_bounds.is_some() {
            return invalid(raw, "integer_bounds is only valid for integer observations");
        }
        if !observation.allowed_render.iter().any(|r| r == "unset") {
            return invalid(
                raw,
                "allowed_render must include the \"unset\" rendering token",
            );
        }
        if value_type == "bool" {
            let mut expected: Vec<&str> = vec!["true", "false", "unset"];
            expected.sort_unstable();
            let mut actual: Vec<&str> = observation
                .allowed_render
                .iter()
                .map(String::as_str)
                .collect();
            actual.sort_unstable();
            if actual != expected {
                return invalid(
                    raw,
                    "bool observations must set allowed_render to [\"true\", \"false\", \"unset\"]",
                );
            }
        }
    }

    // §6.3.3 cardinality + §6.3.6 status legality.
    let mut unset_count = 0usize;
    let mut unrecognized_count = 0usize;
    for outcome in &raw.outcomes {
        match &outcome.match_spec {
            MatchSpec::Unset(flag) => {
                unset_count += 1;
                if !flag || outcome.status != "unknown" {
                    return invalid(
                        raw,
                        "unset outcomes must be `\"unset\": true` with status unknown",
                    );
                }
            }
            MatchSpec::Unrecognized(flag) => {
                unrecognized_count += 1;
                if !flag || outcome.status != "unknown" {
                    return invalid(
                        raw,
                        "unrecognized outcomes must be `\"unrecognized\": true` with status unknown",
                    );
                }
            }
            MatchSpec::Equals { value } => {
                if !matches!(outcome.status.as_str(), "pass" | "finding") {
                    return invalid(raw, "equals outcomes allow only pass or finding status");
                }
                validate_match_value(raw, observation, value)?;
            }
            MatchSpec::AnyOf { values } => {
                if !matches!(outcome.status.as_str(), "pass" | "finding") {
                    return invalid(raw, "any_of outcomes allow only pass or finding status");
                }
                if values.is_empty() {
                    return invalid(raw, "any_of must list at least one value");
                }
                for value in values {
                    validate_match_value(raw, observation, value)?;
                }
            }
            MatchSpec::IntRange { min, max } => {
                if !matches!(outcome.status.as_str(), "pass" | "finding") {
                    return invalid(raw, "int_range outcomes allow only pass or finding status");
                }
                if value_type != "integer" {
                    return invalid(raw, "int_range applies only to integer observations");
                }
                let bounds = observation.integer_bounds.expect("validated above");
                let low = min.unwrap_or(bounds.min);
                let high = max.unwrap_or(bounds.max);
                if low > high {
                    return invalid(raw, "int_range min must be <= max");
                }
                if low < bounds.min || high > bounds.max {
                    return invalid(raw, "int_range must lie within integer_bounds");
                }
            }
        }
    }
    if unset_count != 1 || unrecognized_count != 1 {
        return invalid(
            raw,
            "exactly one unset and exactly one unrecognized outcome are required",
        );
    }

    // §6.3.4 exhaustiveness + §6.3.5 overlap freedom.
    match value_type {
        "enum" => validate_enum_partition(raw),
        "bool" => validate_bool_partition(raw),
        _ => validate_integer_partition(raw),
    }
}

fn validate_match_value(
    raw: &RawRule,
    observation: &Observation,
    value: &MatchValue,
) -> Result<(), RuleValidationError> {
    match (observation.value_type.as_str(), value) {
        ("enum", MatchValue::Str(text)) => {
            let in_domain = text != "unset" && observation.allowed_render.iter().any(|r| r == text);
            if !in_domain {
                return invalid(
                    raw,
                    "match string values must be in allowed_render (excluding \"unset\")",
                );
            }
        }
        ("bool", MatchValue::Bool(_)) => {}
        ("integer", MatchValue::Int(number)) => {
            let bounds = observation.integer_bounds.expect("validated above");
            if *number < bounds.min || *number > bounds.max {
                return invalid(raw, "match integer values must lie within integer_bounds");
            }
        }
        _ => return invalid(raw, "match value type must agree with observation.type"),
    }
    Ok(())
}

fn value_sets(raw: &RawRule) -> Vec<Vec<MatchValue>> {
    raw.outcomes
        .iter()
        .filter_map(|outcome| match &outcome.match_spec {
            MatchSpec::Equals { value } => Some(vec![value.clone()]),
            MatchSpec::AnyOf { values } => Some(values.clone()),
            _ => None,
        })
        .collect()
}

fn validate_enum_partition(raw: &RawRule) -> Result<(), RuleValidationError> {
    let domain: Vec<&str> = raw
        .observation
        .allowed_render
        .iter()
        .map(String::as_str)
        .filter(|render| *render != "unset")
        .collect();
    let sets = value_sets(raw);
    let mut seen: Vec<&str> = Vec::new();
    for set in &sets {
        for value in set {
            let MatchValue::Str(text) = value else {
                return invalid(raw, "enum match values must be strings");
            };
            if seen.contains(&text.as_str()) {
                return invalid(
                    raw,
                    "value-match outcomes overlap; evaluation must be order-independent",
                );
            }
            seen.push(text);
        }
    }
    for value in &domain {
        if !seen.contains(value) {
            return invalid(
                raw,
                "value-match outcomes are not exhaustive over the enum domain",
            );
        }
    }
    Ok(())
}

fn validate_bool_partition(raw: &RawRule) -> Result<(), RuleValidationError> {
    let sets = value_sets(raw);
    let mut seen: Vec<bool> = Vec::new();
    for set in &sets {
        for value in set {
            let MatchValue::Bool(flag) = value else {
                return invalid(raw, "bool match values must be booleans");
            };
            if seen.contains(flag) {
                return invalid(
                    raw,
                    "value-match outcomes overlap; evaluation must be order-independent",
                );
            }
            seen.push(*flag);
        }
    }
    if !(seen.contains(&true) && seen.contains(&false)) {
        return invalid(raw, "bool outcomes must be exhaustive over true and false");
    }
    Ok(())
}

fn validate_integer_partition(raw: &RawRule) -> Result<(), RuleValidationError> {
    let bounds = raw.observation.integer_bounds.expect("validated above");
    // Every value-matching outcome becomes one or more closed intervals.
    let mut intervals: Vec<(i64, i64)> = Vec::new();
    for outcome in &raw.outcomes {
        match &outcome.match_spec {
            MatchSpec::Equals {
                value: MatchValue::Int(number),
            } => {
                intervals.push((*number, *number));
            }
            MatchSpec::AnyOf { values } => {
                for value in values {
                    let MatchValue::Int(number) = value else {
                        return invalid(raw, "integer match values must be integers");
                    };
                    intervals.push((*number, *number));
                }
            }
            MatchSpec::IntRange { min, max } => {
                intervals.push((min.unwrap_or(bounds.min), max.unwrap_or(bounds.max)));
            }
            _ => {}
        }
    }
    intervals.sort_unstable();
    // i128 bookkeeping: `i64::MAX + 1` must be representable so a value at
    // the domain ceiling is never double-counted as both "the last covered
    // value" and "one past it" — the failure mode of doing this arithmetic
    // (and its saturation) in i64.
    let mut expected_next: i128 = i128::from(bounds.min);
    for (low, high) in &intervals {
        let low = i128::from(*low);
        let high = i128::from(*high);
        if low < expected_next {
            return invalid(
                raw,
                "integer outcomes overlap; evaluation must be order-independent",
            );
        }
        if low > expected_next {
            return invalid(
                raw,
                "integer outcomes do not cover integer_bounds exhaustively",
            );
        }
        expected_next = high + 1;
    }
    if intervals.is_empty() || expected_next <= i128::from(bounds.max) {
        return invalid(
            raw,
            "integer outcomes do not cover integer_bounds exhaustively",
        );
    }
    Ok(())
}

fn invalid<T>(raw: &RawRule, message: &str) -> Result<T, RuleValidationError> {
    Err(RuleValidationError(format!("rule {} {message}", raw.id)))
}

/// All bundled rules, sorted by id. Panics only on a corrupt embed, which
/// `cargo test` catches before any release build ships.
pub fn load_rules() -> Vec<ValidatedRule> {
    let mut rules: Vec<ValidatedRule> = EMBEDDED_RULES
        .iter()
        .map(|(path, text)| {
            let raw: RawRule = serde_json::from_str(text)
                .unwrap_or_else(|error| panic!("embedded rule {path} is invalid JSON: {error}"));
            ValidatedRule::try_from_raw(raw)
                .unwrap_or_else(|error| panic!("embedded rule {path} failed validation: {error}"))
        })
        .collect();
    rules.sort_by(|left, right| left.raw().id.cmp(&right.raw().id));
    rules
}

pub fn ruleset_version() -> String {
    let v: serde_json::Value =
        serde_json::from_str(RULESET_JSON).expect("embedded ruleset.json is valid");
    v["ruleset_version"]
        .as_str()
        .expect("ruleset_version present")
        .to_string()
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    fn bundled_rule_text() -> &'static str {
        EMBEDDED_RULES
            .iter()
            .find(|(path, _)| *path == "codex/history-persist-01.json")
            .expect("codex rule is embedded")
            .1
    }

    fn raw_rule() -> RawRule {
        serde_json::from_str(bundled_rule_text()).unwrap()
    }

    fn raw_with(mutate: impl FnOnce(&mut serde_json::Value)) -> RawRule {
        let mut json: serde_json::Value = serde_json::from_str(bundled_rule_text()).unwrap();
        mutate(&mut json);
        serde_json::from_value(json).expect("corpus mutation still deserializes")
    }

    /// A synthetic integer-observation rule derived from the bundled enum
    /// rule: same shape (2 value outcomes + unset + unrecognized), retyped
    /// to `integer` with bounds `[0, 90]` split as `[0,29]` / `[30,90]`.
    fn integer_raw_with(mutate: impl FnOnce(&mut serde_json::Value)) -> RawRule {
        let mut json: serde_json::Value = serde_json::from_str(bundled_rule_text()).unwrap();
        json["observation"]["type"] = serde_json::json!("integer");
        json["observation"]["allowed_render"] = serde_json::json!(["unset"]);
        json["observation"]["integer_bounds"] = serde_json::json!({"min": 0, "max": 90});
        json["outcomes"][0]["match"] = serde_json::json!({"int_range": {"min": 0, "max": 29}});
        json["outcomes"][1]["match"] = serde_json::json!({"int_range": {"min": 30, "max": 90}});
        mutate(&mut json);
        serde_json::from_value(json).expect("integer corpus mutation still deserializes")
    }

    fn assert_rejected(rule: RawRule, needle: &str) {
        let error = ValidatedRule::try_from_raw(rule).expect_err("must be rejected");
        assert!(
            error.0.contains(needle),
            "error {:?} should mention {needle:?}",
            error.0
        );
    }

    #[test]
    fn bundled_rule_passes_validation() {
        assert!(ValidatedRule::try_from_raw(raw_rule()).is_ok());
    }

    // §6.3.1 type agreement
    #[test]
    fn equals_bool_on_enum_observation_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][0]["match"] = serde_json::json!({"equals": {"value": true}})
            }),
            "match value type",
        );
    }
    #[test]
    fn int_range_on_enum_observation_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][0]["match"] = serde_json::json!({"int_range": {"min": 0, "max": 1}})
            }),
            "int_range",
        );
    }

    // §6.3.2 domain membership
    #[test]
    fn equals_value_outside_allowed_render_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][0]["match"] = serde_json::json!({"equals": {"value": "archive"}})
            }),
            "allowed_render",
        );
    }
    #[test]
    fn equals_value_unset_string_is_rejected() {
        // "unset" is the unset-rendering token, not a matchable domain value.
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][0]["match"] = serde_json::json!({"equals": {"value": "unset"}})
            }),
            "allowed_render",
        );
    }

    // §6.3.3 cardinality
    #[test]
    fn missing_unrecognized_catch_all_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                let outcomes = j["outcomes"].as_array_mut().unwrap();
                outcomes.retain(|o| o["match"].get("unrecognized").is_none());
            }),
            "unrecognized",
        );
    }
    #[test]
    fn duplicate_unset_outcome_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                let unset = j["outcomes"][2].clone();
                j["outcomes"].as_array_mut().unwrap().push(unset);
            }),
            "exactly one",
        );
    }

    // §6.3.4 exhaustiveness
    #[test]
    fn uncovered_enum_domain_value_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                // Domain becomes {save-all, none, keep-latest} but no outcome
                // matches keep-latest.
                j["observation"]["allowed_render"] =
                    serde_json::json!(["save-all", "none", "keep-latest", "unset"]);
            }),
            "exhaustive",
        );
    }

    // §6.3.5 overlap freedom
    #[test]
    fn overlapping_value_outcomes_are_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][1]["match"] =
                    serde_json::json!({"any_of": {"values": ["save-all", "none"]}})
            }),
            "overlap",
        );
    }

    // §6.3.6 status legality
    #[test]
    fn unset_with_pass_status_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][2]["status"] = serde_json::json!("pass");
                j["outcomes"][2]["unknown_reason"] = serde_json::Value::Null;
                j["outcomes"][2]["verify_url"] = serde_json::Value::Null;
                j["outcomes"][2]["confidence"] = serde_json::json!("high");
            }),
            "unset",
        );
    }
    #[test]
    fn value_match_with_unknown_status_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["outcomes"][0]["status"] = serde_json::json!("unknown");
                j["outcomes"][0]["confidence"] = serde_json::Value::Null;
                j["outcomes"][0]["unknown_reason"] = serde_json::json!("x");
            }),
            "equals",
        );
    }

    // Integer-rule corpus (§6.3.1, §6.3.4, §6.3.5 for the integer domain).
    #[test]
    fn full_integer_coverage_is_accepted() {
        assert!(ValidatedRule::try_from_raw(integer_raw_with(|_| {})).is_ok());
    }
    #[test]
    fn int_range_escaping_integer_bounds_is_rejected() {
        assert_rejected(
            integer_raw_with(|j| {
                j["outcomes"][0]["match"] =
                    serde_json::json!({"int_range": {"min": -5, "max": 10}});
            }),
            "integer_bounds",
        );
    }
    #[test]
    fn inverted_int_range_is_rejected() {
        assert_rejected(
            integer_raw_with(|j| {
                j["outcomes"][0]["match"] =
                    serde_json::json!({"int_range": {"min": 50, "max": 10}});
            }),
            "min must be <= max",
        );
    }
    #[test]
    fn gap_in_integer_coverage_is_rejected() {
        assert_rejected(
            integer_raw_with(|j| {
                j["outcomes"][0]["match"] = serde_json::json!({"int_range": {"min": 0, "max": 29}});
                j["outcomes"][1]["match"] =
                    serde_json::json!({"int_range": {"min": 40, "max": 90}});
            }),
            "cover",
        );
    }
    #[test]
    fn overlapping_integer_ranges_are_rejected() {
        assert_rejected(
            integer_raw_with(|j| {
                j["outcomes"][0]["match"] = serde_json::json!({"int_range": {"min": 0, "max": 40}});
                j["outcomes"][1]["match"] =
                    serde_json::json!({"int_range": {"min": 30, "max": 90}});
            }),
            "overlap",
        );
    }
    #[test]
    fn non_unset_allowed_render_for_integer_observation_is_rejected() {
        assert_rejected(
            integer_raw_with(|j| {
                j["observation"]["allowed_render"] = serde_json::json!(["unset", "extra"]);
            }),
            "allowed_render",
        );
    }

    /// A synthetic integer rule for `i64` boundary corpus cases: the
    /// unset/unrecognized outcomes are kept, but the value-matching
    /// outcomes are replaced wholesale by `value_matches` (each cloned from
    /// the bundled rule's first outcome, with only `match` substituted), and
    /// `integer_bounds` is set to `[min, max]` directly.
    fn integer_boundary_raw(min: i64, max: i64, value_matches: &[serde_json::Value]) -> RawRule {
        let mut json: serde_json::Value = serde_json::from_str(bundled_rule_text()).unwrap();
        json["observation"]["type"] = serde_json::json!("integer");
        json["observation"]["allowed_render"] = serde_json::json!(["unset"]);
        json["observation"]["integer_bounds"] = serde_json::json!({"min": min, "max": max});
        let template = json["outcomes"][0].clone();
        let unset = json["outcomes"][2].clone();
        let unrecognized = json["outcomes"][3].clone();
        let mut outcomes: Vec<serde_json::Value> = value_matches
            .iter()
            .map(|match_spec| {
                let mut outcome = template.clone();
                outcome["match"] = match_spec.clone();
                outcome
            })
            .collect();
        outcomes.push(unset);
        outcomes.push(unrecognized);
        json["outcomes"] = serde_json::Value::Array(outcomes);
        serde_json::from_value(json).expect("integer boundary corpus mutation still deserializes")
    }

    // i64::MAX boundary: `saturating_add(1)` on the terminal interval must
    // not be mistaken for "still short of bounds.max" (§6.3.4 soundness at
    // the domain ceiling).
    #[test]
    fn single_interval_reaching_i64_max_is_accepted() {
        assert!(
            ValidatedRule::try_from_raw(integer_boundary_raw(
                0,
                i64::MAX,
                &[serde_json::json!({"int_range": {"min": 0, "max": i64::MAX}})],
            ))
            .is_ok()
        );
    }
    #[test]
    fn two_interval_split_reaching_i64_max_is_accepted() {
        let midpoint = i64::MAX / 2;
        assert!(
            ValidatedRule::try_from_raw(integer_boundary_raw(
                0,
                i64::MAX,
                &[
                    serde_json::json!({"int_range": {"min": 0, "max": midpoint}}),
                    serde_json::json!({"int_range": {"min": midpoint + 1, "max": i64::MAX}}),
                ],
            ))
            .is_ok()
        );
    }
    #[test]
    fn gap_just_below_i64_max_is_still_rejected() {
        assert_rejected(
            integer_boundary_raw(
                0,
                i64::MAX,
                &[serde_json::json!({"int_range": {"min": 0, "max": i64::MAX - 1}})],
            ),
            "cover",
        );
    }
    #[test]
    fn overlap_exactly_at_i64_max_is_rejected() {
        // Both outcomes match the value i64::MAX: {0, i64::MAX} entirely
        // subsumes {i64::MAX, i64::MAX}. i64 saturation must not swallow
        // this overlap at the domain ceiling.
        assert_rejected(
            integer_boundary_raw(
                0,
                i64::MAX,
                &[
                    serde_json::json!({"int_range": {"min": 0, "max": i64::MAX}}),
                    serde_json::json!({"int_range": {"min": i64::MAX, "max": i64::MAX}}),
                ],
            ),
            "overlap",
        );
    }

    // Dedicated one-liners the reviewer flagged as only indirectly covered.
    #[test]
    fn unset_flag_false_is_rejected() {
        assert_rejected(
            raw_with(|j| j["outcomes"][2]["match"] = serde_json::json!({"unset": false})),
            "unset",
        );
    }
    #[test]
    fn int_range_on_bool_observation_is_rejected() {
        assert_rejected(
            raw_with(|j| {
                j["observation"]["type"] = serde_json::json!("bool");
                j["observation"]["allowed_render"] = serde_json::json!(["true", "false", "unset"]);
                j["outcomes"][0]["match"] = serde_json::json!({"int_range": {"min": 0, "max": 1}});
            }),
            "int_range",
        );
    }
}
