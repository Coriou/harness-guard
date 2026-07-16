//! Rule evaluation (§5.4 status model). Degradation is conservative and
//! total: every path yields a FindingRecord; nothing is dropped or
//! silently passed.
use crate::parse::{ExtractedValue, ParseFailure};
use crate::readfs::RefusalReason;
use crate::version::version_in_range;
use harness_guard_rules::loader::ValidatedRule;
use harness_guard_rules::report::{Confidence, FindingRecord, Severity, SourceCite, Status};
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

    // Declared unknown conditions take precedence over version bookkeeping.
    match config {
        ConfigState::Unreadable(reason) => {
            return unknown(base, reason.describe().to_string(), rule);
        }
        ConfigState::Unparseable(failure) => {
            return unknown(
                base,
                format!("config not safely parseable: {}", failure.message),
                rule,
            );
        }
        ConfigState::Missing | ConfigState::Parsed(_) => {}
    }

    let in_range = detected_version
        .map(|version| version_in_range(version, &raw.tested_versions))
        .unwrap_or(false);
    let (observation, indicative) = observe(rule, config);

    if !in_range {
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
        let message = if matches!(indicative.kind, IndicativeKind::Unrecognized) {
            "Unverified — last-known rule indicates the configured value cannot be interpreted safely. Observed: unrecognized value (raw value withheld).".to_string()
        } else {
            format!(
                "Unverified — last-known rule indicates: {} Observed: {}.",
                indicative.message,
                observation
                    .as_deref()
                    .unwrap_or("unrecognized value (raw value withheld)")
            )
        };
        return checked(FindingRecord {
            status: Status::StaleRuleset,
            severity: None,
            confidence: None,
            message,
            observation,
            remediation: None,
            stale_reason: Some(stale_reason),
            ..base
        });
    }

    match indicative.kind {
        IndicativeKind::Pass => checked(FindingRecord {
            status: Status::Pass,
            severity: None,
            confidence: Some(Confidence::High),
            message: indicative.message,
            observation,
            remediation: None,
            ..base
        }),
        IndicativeKind::Finding => checked(FindingRecord {
            status: Status::Finding,
            severity: Some(Severity::Warning),
            confidence: Some(Confidence::High),
            message: indicative.message,
            observation,
            remediation: rule
                .raw()
                .outcomes
                .iter()
                .find(|outcome| outcome.status == "finding")
                .and_then(|outcome| outcome.remediation.clone()),
            ..base
        }),
        IndicativeKind::Unset => unknown_with_observation(
            base,
            "history.persistence is unset in the user-level config; uninspected system, profile, trusted-project, or CLI layers may determine the effective value."
                .to_string(),
            observation,
            rule,
        ),
        IndicativeKind::Unrecognized => unknown(
            base,
            "history.persistence is set to an unrecognized value — raw values are never displayed"
                .to_string(),
            rule,
        ),
    }
}

enum IndicativeKind {
    Pass,
    Finding,
    Unset,
    Unrecognized,
}

struct Indicative {
    kind: IndicativeKind,
    message: String,
}

/// Allowlisted observation rendering and the outcome indicated by that value.
/// Unrecognized raw values are dropped inside this function.
fn observe(rule: &ValidatedRule, config: &ConfigState) -> (Option<String>, Indicative) {
    let raw = rule.raw();
    let key = &raw.observation.key;
    let value = match config {
        ConfigState::Parsed(values) => values.get(key).cloned().unwrap_or(ExtractedValue::Unset),
        ConfigState::Missing => ExtractedValue::Unset,
        ConfigState::Unreadable(_) | ConfigState::Unparseable(_) => ExtractedValue::Unset,
    };
    let outcome_message = |status: &str| {
        raw.outcomes
            .iter()
            .find(|outcome| outcome.status == status)
            .map(|outcome| outcome.message.clone())
            .unwrap_or_default()
    };

    match value {
        ExtractedValue::Str(value) if value == "none" => (
            Some(format!("{key} = \"none\"")),
            Indicative {
                kind: IndicativeKind::Pass,
                message: outcome_message("pass"),
            },
        ),
        ExtractedValue::Str(value) if value == "save-all" => (
            Some(format!("{key} = \"save-all\"")),
            Indicative {
                kind: IndicativeKind::Finding,
                message: outcome_message("finding"),
            },
        ),
        ExtractedValue::Unset => (
            Some(format!("{key} unset in user config")),
            Indicative {
                kind: IndicativeKind::Unset,
                message: raw
                    .outcomes
                    .iter()
                    .find(|outcome| {
                        outcome.status == "unknown" && outcome.when.contains("is unset")
                    })
                    .map(|outcome| outcome.message.clone())
                    .unwrap_or_default(),
            },
        ),
        ExtractedValue::Str(_)
        | ExtractedValue::Bool(_)
        | ExtractedValue::Int(_)
        | ExtractedValue::Other => (
            None,
            Indicative {
                kind: IndicativeKind::Unrecognized,
                message: String::new(),
            },
        ),
    }
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

fn unknown(base: FindingRecord, reason: String, rule: &ValidatedRule) -> FindingRecord {
    unknown_with_observation(base, reason, None, rule)
}

fn unknown_with_observation(
    base: FindingRecord,
    reason: String,
    observation: Option<String>,
    rule: &ValidatedRule,
) -> FindingRecord {
    let verify_url = rule
        .raw()
        .outcomes
        .iter()
        .find(|outcome| outcome.status == "unknown")
        .and_then(|outcome| outcome.verify_url.clone());
    checked(FindingRecord {
        status: Status::Unknown,
        severity: None,
        confidence: None,
        evidence_class: None,
        message: format!("Cannot determine history persistence posture: {reason}"),
        observation,
        remediation: None,
        source: None,
        unknown_reason: Some(reason),
        verify_url,
        ..base
    })
}

fn checked(finding: FindingRecord) -> FindingRecord {
    finding
        .validate()
        .expect("core evaluation must construct a schema-valid finding record");
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
}
