//! Declarative rule evaluation (spec §6). Rule JSON drives evaluation through
//! the closed match-primitive set; precedence, message templates, and
//! observation rendering are engine-fixed. Loader validation (§6.3) proves at
//! load time that evaluation is total and order-independent, so the lookups
//! here cannot fall through. Degradation is conservative: every path yields a
//! schema-valid FindingRecord.
use crate::parse::{ExtractedValue, ParseFailure};
use crate::readfs::RefusalReason;
use crate::version::version_in_range;
use harness_guard_rules::loader::ValidatedRule;
use harness_guard_rules::report::{Confidence, FindingRecord, Severity, SourceCite, Status};
use harness_guard_rules::schema::{MatchSpec, MatchValue, RawOutcome, RawRule};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum ConfigState {
    /// Tool detected but no user config file. Other layers may still supply
    /// the effective value, so this remains unknown.
    Missing,
    Unreadable(RefusalReason),
    Unparseable(ParseFailure),
    Parsed(BTreeMap<String, ExtractedValue>),
}

pub fn evaluate_rule(
    rule: &ValidatedRule,
    config: &ConfigState,
    detected_version: Option<&str>,
) -> FindingRecord {
    let base = base_record(rule);
    let raw = rule.raw();

    // §6.4.1 — declared unknown conditions beat version bookkeeping.
    match config {
        ConfigState::Unreadable(reason) => {
            return engine_unknown(base, rule, reason.describe().to_string());
        }
        ConfigState::Unparseable(failure) => {
            return engine_unknown(
                base,
                rule,
                format!("config not safely parseable: {}", failure.message),
            );
        }
        ConfigState::Missing | ConfigState::Parsed(_) => {}
    }

    // §6.4.2 — extract the typed value, select the unique matching outcome.
    let value = extracted_value(raw, config);
    let matched = select_outcome(raw, &value);
    let observation = render_observation(raw, &value, matched);

    // §6.4.3 — out-of-range or undetected version wraps the matched outcome.
    let in_range = detected_version
        .map(|version| version_in_range(version, &raw.tested_versions))
        .unwrap_or(false);
    if !in_range {
        return stale(base, raw, matched, observation, detected_version);
    }

    // §6.4.4 — emit the matched outcome with the §5.7 rendering.
    emit(base, rule, matched, observation)
}

fn extracted_value(raw: &RawRule, config: &ConfigState) -> ExtractedValue {
    match config {
        ConfigState::Parsed(values) => values
            .get(&raw.observation.key)
            .cloned()
            .unwrap_or(ExtractedValue::Unset),
        // Missing config: other layers may supply the value — unset.
        // Unreadable/Unparseable are unreachable (early return) but total.
        _ => ExtractedValue::Unset,
    }
}

/// §6.2: present-but-outside-domain values (type mismatches, Other, strings
/// outside allowed_render, out-of-integer_bounds integers) are unrecognized
/// BEFORE any value matching, so an open-ended int_range can never claim an
/// out-of-domain integer.
fn in_domain(raw: &RawRule, value: &ExtractedValue) -> bool {
    match (raw.observation.value_type.as_str(), value) {
        (_, ExtractedValue::Unset) => true,
        ("enum", ExtractedValue::Str(text)) => {
            text != "unset" && raw.observation.allowed_render.iter().any(|r| r == text)
        }
        ("bool", ExtractedValue::Bool(_)) => true,
        ("integer", ExtractedValue::Int(number)) => raw
            .observation
            .integer_bounds
            .is_some_and(|bounds| *number >= bounds.min && *number <= bounds.max),
        _ => false,
    }
}

fn select_outcome<'r>(raw: &'r RawRule, value: &ExtractedValue) -> &'r RawOutcome {
    if !in_domain(raw, value) {
        return unrecognized_outcome(raw);
    }
    if matches!(value, ExtractedValue::Unset) {
        return unset_outcome(raw);
    }
    raw.outcomes
        .iter()
        .find(|outcome| match_fires(&outcome.match_spec, value))
        // Unreachable for validated rules (§6.3 exhaustiveness); stay total
        // and conservative rather than panicking on a hostile forked ruleset.
        .unwrap_or_else(|| unrecognized_outcome(raw))
}

fn match_fires(spec: &MatchSpec, value: &ExtractedValue) -> bool {
    match (spec, value) {
        (MatchSpec::Equals { value: expected }, observed) => value_eq(expected, observed),
        (MatchSpec::AnyOf { values }, observed) => {
            values.iter().any(|expected| value_eq(expected, observed))
        }
        (MatchSpec::IntRange { min, max }, ExtractedValue::Int(number)) => {
            min.is_none_or(|low| *number >= low) && max.is_none_or(|high| *number <= high)
        }
        _ => false,
    }
}

fn value_eq(expected: &MatchValue, observed: &ExtractedValue) -> bool {
    match (expected, observed) {
        (MatchValue::Str(left), ExtractedValue::Str(right)) => left == right,
        (MatchValue::Bool(left), ExtractedValue::Bool(right)) => left == right,
        (MatchValue::Int(left), ExtractedValue::Int(right)) => left == right,
        _ => false,
    }
}

fn unset_outcome(raw: &RawRule) -> &RawOutcome {
    raw.outcomes
        .iter()
        .find(|outcome| matches!(outcome.match_spec, MatchSpec::Unset(_)))
        .expect("validated rules carry exactly one unset outcome (§6.3.3)")
}

fn unrecognized_outcome(raw: &RawRule) -> &RawOutcome {
    raw.outcomes
        .iter()
        .find(|outcome| matches!(outcome.match_spec, MatchSpec::Unrecognized(_)))
        .expect("validated rules carry exactly one unrecognized outcome (§6.3.3)")
}

/// §5.7: observations render ONLY from the parsed typed value, never source
/// text. Enum strings passed the domain check; integers passed the bounds
/// check — re-serializing them cannot leak arbitrary content.
fn render_observation(
    raw: &RawRule,
    value: &ExtractedValue,
    matched: &RawOutcome,
) -> Option<String> {
    let key = &raw.observation.key;
    match matched.match_spec {
        MatchSpec::Unrecognized(_) => None,
        MatchSpec::Unset(_) => Some(format!("{key} unset in user config")),
        _ => match value {
            ExtractedValue::Str(text) => Some(format!("{key} = \"{text}\"")),
            ExtractedValue::Bool(flag) => Some(format!("{key} = {flag}")),
            ExtractedValue::Int(number) => Some(format!("{key} = {number}")),
            ExtractedValue::Unset | ExtractedValue::Other => None,
        },
    }
}

fn emit(
    base: FindingRecord,
    rule: &ValidatedRule,
    outcome: &RawOutcome,
    observation: Option<String>,
) -> FindingRecord {
    match outcome.status.as_str() {
        "pass" => checked(FindingRecord {
            status: Status::Pass,
            severity: None,
            confidence: confidence_of(outcome),
            message: outcome.message.clone(),
            observation,
            remediation: None,
            ..base
        }),
        "finding" => checked(FindingRecord {
            status: Status::Finding,
            severity: severity_of(outcome),
            confidence: confidence_of(outcome),
            message: outcome.message.clone(),
            observation,
            remediation: outcome.remediation.clone(),
            ..base
        }),
        // unknown — the unset and unrecognized outcomes (§6.3.6).
        _ => {
            let reason = outcome.unknown_reason.clone().unwrap_or_default();
            unknown_record(base, rule, reason, observation, outcome.verify_url.clone())
        }
    }
}

/// Engine-level unknowns (unreadable/unparseable config). verify_url comes
/// from the rule's single unset outcome — deterministic by §6.3.3.
fn engine_unknown(base: FindingRecord, rule: &ValidatedRule, reason: String) -> FindingRecord {
    let verify_url = unset_outcome(rule.raw()).verify_url.clone();
    unknown_record(base, rule, reason, None, verify_url)
}

/// The engine's only unknown-message template (plan assumption 1):
/// "Cannot determine {unknown_subject}: {reason}". No other interpolation of
/// rule text exists anywhere in the engine.
fn unknown_record(
    base: FindingRecord,
    rule: &ValidatedRule,
    reason: String,
    observation: Option<String>,
    verify_url: Option<String>,
) -> FindingRecord {
    checked(FindingRecord {
        status: Status::Unknown,
        severity: None,
        confidence: None,
        evidence_class: None,
        message: format!("Cannot determine {}: {reason}", rule.raw().unknown_subject),
        observation,
        remediation: None,
        source: None,
        unknown_reason: Some(reason),
        verify_url,
        stale_reason: None,
        ..base
    })
}

fn stale(
    base: FindingRecord,
    raw: &RawRule,
    matched: &RawOutcome,
    observation: Option<String>,
    detected_version: Option<&str>,
) -> FindingRecord {
    let stale_reason = match detected_version {
        None => "tool version not detected — no version marker found on PATH".to_string(),
        Some(version) => format!(
            "detected version {version} is outside every tested range (max tested {})",
            raw.tested_versions
                .iter()
                .map(|tested| tested.max.as_str())
                .max()
                .unwrap_or("?")
        ),
    };
    // The unrecognized+stale safe fallback phrasing is preserved verbatim
    // (adjudicated review finding 9; spec §6.4.3).
    let message = if matches!(matched.match_spec, MatchSpec::Unrecognized(_)) {
        "Unverified — last-known rule indicates the configured value cannot be interpreted safely. Observed: unrecognized value (raw value withheld).".to_string()
    } else {
        format!(
            "Unverified — last-known rule indicates: {} Observed: {}.",
            matched.message,
            observation
                .as_deref()
                .unwrap_or("unrecognized value (raw value withheld)")
        )
    };
    checked(FindingRecord {
        status: Status::StaleRuleset,
        severity: None,
        confidence: None,
        message,
        observation,
        remediation: None,
        stale_reason: Some(stale_reason),
        ..base
    })
}

fn confidence_of(outcome: &RawOutcome) -> Option<Confidence> {
    outcome
        .confidence
        .as_deref()
        .map(|confidence| match confidence {
            "low" => Confidence::Low,
            "medium" => Confidence::Medium,
            _ => Confidence::High,
        })
}

fn severity_of(outcome: &RawOutcome) -> Option<Severity> {
    outcome.severity.as_deref().map(|severity| {
        if severity == "info" {
            Severity::Info
        } else {
            Severity::Warning
        }
    })
}

fn base_record(rule: &ValidatedRule) -> FindingRecord {
    let raw = rule.raw();
    let primary_source = rule.primary_source();
    let tested_version = &raw.tested_versions[0];
    FindingRecord {
        rule_id: raw.id.clone(),
        status: Status::Unknown,
        severity: None,
        confidence: None,
        evidence_class: Some(primary_source.evidence_class.clone()),
        message: String::new(),
        observation: None,
        remediation: None,
        source: Some(SourceCite {
            url: primary_source.url.clone(),
            retrieved: primary_source.retrieved.clone(),
        }),
        valid_from: Some(tested_version.min.clone()),
        valid_until: Some(tested_version.max.clone()),
        limitations: raw.limitations.clone(),
        unknown_reason: None,
        verify_url: None,
        stale_reason: None,
    }
}

fn checked(finding: FindingRecord) -> FindingRecord {
    finding
        .validate()
        .expect("engine must construct a schema-valid finding record");
    finding
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ExtractedValue;
    use harness_guard_rules::loader::load_rules;
    use harness_guard_rules::report::Status;
    use std::collections::BTreeMap;

    fn rule() -> harness_guard_rules::loader::ValidatedRule {
        load_rules().into_iter().next().unwrap()
    }

    fn parsed(value: Option<ExtractedValue>) -> ConfigState {
        let mut values = BTreeMap::new();
        values.insert(
            "history.persistence".to_string(),
            value.unwrap_or(ExtractedValue::Unset),
        );
        ConfigState::Parsed(values)
    }

    #[test]
    fn none_in_range_passes_with_citation() {
        let finding = evaluate_rule(
            &rule(),
            &parsed(Some(ExtractedValue::Str("none".into()))),
            Some("0.144.5"),
        );
        assert_eq!(finding.status, Status::Pass);
        assert!(finding.source.is_some(), "pass requires a citation (§5.4)");
        assert_eq!(
            finding.observation.as_deref(),
            Some("history.persistence = \"none\"")
        );
        assert_eq!(finding.valid_until.as_deref(), Some("0.144.5"));
    }

    #[test]
    fn unset_in_range_is_unknown_because_other_layers_are_uninspected() {
        let finding = evaluate_rule(&rule(), &parsed(None), Some("0.144.5"));
        assert_eq!(finding.status, Status::Unknown);
        assert_eq!(finding.severity, None);
        assert_eq!(
            finding.observation.as_deref(),
            Some("history.persistence unset in user config")
        );
        assert!(finding.remediation.is_none());
        assert!(finding.source.is_none());
        assert!(
            finding
                .unknown_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("uninspected system"))
        );
    }

    #[test]
    fn explicit_save_all_is_warning_finding() {
        let finding = evaluate_rule(
            &rule(),
            &parsed(Some(ExtractedValue::Str("save-all".into()))),
            Some("0.144.5"),
        );
        assert_eq!(finding.status, Status::Finding);
        assert_eq!(
            finding.observation.as_deref(),
            Some("history.persistence = \"save-all\"")
        );
    }

    #[test]
    fn unrecognized_value_is_unknown_and_never_echoed() {
        let finding = evaluate_rule(
            &rule(),
            &parsed(Some(ExtractedValue::Str("archive".into()))),
            Some("0.144.5"),
        );
        assert_eq!(finding.status, Status::Unknown);
        assert!(finding.severity.is_none() && finding.confidence.is_none());
        assert!(finding.unknown_reason.is_some());
        assert!(finding.observation.is_none());
        let json = serde_json::to_string(&finding).unwrap();
        assert!(
            !json.contains("archive"),
            "raw value leaked into the record"
        );
    }

    #[test]
    fn non_string_value_is_unknown() {
        let finding = evaluate_rule(
            &rule(),
            &parsed(Some(ExtractedValue::Other)),
            Some("0.144.5"),
        );
        assert_eq!(finding.status, Status::Unknown);
    }

    #[test]
    fn missing_config_with_tool_detected_is_unknown() {
        let finding = evaluate_rule(&rule(), &ConfigState::Missing, Some("0.144.5"));
        assert_eq!(finding.status, Status::Unknown);
    }

    #[test]
    fn unreadable_config_is_unknown() {
        let finding = evaluate_rule(
            &rule(),
            &ConfigState::Unreadable(crate::readfs::RefusalReason::PermissionDenied),
            Some("0.144.5"),
        );
        assert_eq!(finding.status, Status::Unknown);
        assert!(
            finding
                .unknown_reason
                .as_deref()
                .unwrap()
                .contains("permission")
        );
    }

    #[test]
    fn undetected_version_is_stale_never_pass() {
        let finding = evaluate_rule(
            &rule(),
            &parsed(Some(ExtractedValue::Str("none".into()))),
            None,
        );
        assert_eq!(finding.status, Status::StaleRuleset);
        assert!(
            finding
                .stale_reason
                .as_deref()
                .unwrap()
                .contains("not detected")
        );
        assert!(finding.message.to_lowercase().contains("unverified"));
        assert!(finding.message.contains("history.persistence = \"none\""));
        assert!(finding.source.is_some());
        assert!(finding.severity.is_none() && finding.confidence.is_none());
    }

    #[test]
    fn out_of_range_version_is_stale_never_pass() {
        let finding = evaluate_rule(&rule(), &parsed(None), Some("9.9.9"));
        assert_eq!(finding.status, Status::StaleRuleset);
        assert!(finding.stale_reason.as_deref().unwrap().contains("9.9.9"));
    }

    #[test]
    fn unknown_beats_stale_when_config_unreadable() {
        let finding = evaluate_rule(
            &rule(),
            &ConfigState::Unreadable(crate::readfs::RefusalReason::Symlink),
            None,
        );
        assert_eq!(finding.status, Status::Unknown);
    }

    #[test]
    fn stale_unrecognized_value_uses_safe_nonempty_fallback_without_raw_value() {
        const RAW_VALUE: &str = "hostile-archive-value";
        for detected_version in [None, Some("9.9.9")] {
            let finding = evaluate_rule(
                &rule(),
                &parsed(Some(ExtractedValue::Str(RAW_VALUE.into()))),
                detected_version,
            );
            assert_eq!(finding.status, Status::StaleRuleset);
            assert!(finding.observation.is_none());
            assert!(finding.message.contains("cannot be interpreted safely"));
            assert!(finding.message.contains("raw value withheld"));
            assert!(!finding.message.contains(RAW_VALUE));
            assert!(
                !finding
                    .message
                    .contains("last-known rule indicates:  Observed: n/a")
            );
            let json = serde_json::to_string(&finding).unwrap();
            assert!(!json.contains(RAW_VALUE));
        }
    }

    #[test]
    fn unknown_messages_reproduce_the_golden_template() {
        let finding = evaluate_rule(
            &rule(),
            &ConfigState::Unreadable(crate::readfs::RefusalReason::PermissionDenied),
            Some("0.144.5"),
        );
        assert_eq!(
            finding.message,
            "Cannot determine history persistence posture: config file is not readable (permission denied)"
        );
        let finding = evaluate_rule(&rule(), &parsed(None), Some("0.144.5"));
        assert_eq!(
            finding.message,
            "Cannot determine history persistence posture: history.persistence is unset in the user-level config; uninspected system, profile, trusted-project, or CLI layers may determine the effective value."
        );
        let finding = evaluate_rule(
            &rule(),
            &parsed(Some(ExtractedValue::Str("archive".into()))),
            Some("0.144.5"),
        );
        assert_eq!(
            finding.message,
            "Cannot determine history persistence posture: history.persistence is set to an unrecognized value — raw values are never displayed"
        );
    }

    /// Task 7: `evaluate.rs` is gone, so the cross-comparison this test used
    /// to run (`engine_matches_evaluate_across_the_full_status_matrix`,
    /// Task 6) no longer has a second implementation to compare against —
    /// its job is done. The behavior it protected stays pinned by the
    /// unchanged fixture goldens (`fixtures/`) and CLI insta snapshots
    /// (`crates/harness-guard-cli/tests/snapshots/`), which the Task 7
    /// switchover proved byte-identical, plus the ported unit tests above
    /// covering every status branch (pass/finding/unset/unrecognized/
    /// unreadable/unparseable/stale) for this rule at the in-range version.
    /// These two tests instead pin engine-only coverage the removed matrix
    /// also exercised but the golden fixtures don't reach directly: every
    /// `RefusalReason` variant, and a synthetic `Unparseable` failure, still
    /// degrade to `Status::Unknown` with the right reason text across an
    /// undetected, in-range, and out-of-range detected version each (unknown
    /// beats stale regardless of version).
    #[test]
    fn every_refusal_reason_degrades_to_unknown_regardless_of_version() {
        let reasons = [
            RefusalReason::PermissionDenied,
            RefusalReason::Symlink,
            RefusalReason::NotRegularFile,
            RefusalReason::Oversized,
            RefusalReason::NotUtf8,
            RefusalReason::Io,
        ];
        for reason in reasons {
            for detected_version in [None, Some("0.144.5"), Some("9.9.9")] {
                let finding =
                    evaluate_rule(&rule(), &ConfigState::Unreadable(reason), detected_version);
                assert_eq!(finding.status, Status::Unknown);
                assert_eq!(
                    finding.unknown_reason.as_deref(),
                    Some(reason.describe()),
                    "reason={reason:?} detected_version={detected_version:?}"
                );
            }
        }
    }

    #[test]
    fn unparseable_config_degrades_to_unknown_regardless_of_version() {
        let failure = ParseFailure {
            line: Some(1),
            col: Some(1),
            key_path: None,
            message: "expected an equals sign".to_string(),
        };
        for detected_version in [None, Some("0.144.5"), Some("9.9.9")] {
            let finding = evaluate_rule(
                &rule(),
                &ConfigState::Unparseable(failure.clone()),
                detected_version,
            );
            assert_eq!(finding.status, Status::Unknown);
            assert_eq!(
                finding.unknown_reason.as_deref(),
                Some("config not safely parseable: expected an equals sign"),
                "detected_version={detected_version:?}"
            );
        }
    }
}
