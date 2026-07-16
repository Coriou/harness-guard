//! The scan orchestrator: discovery → bounded read → parse → extract →
//! evaluate each rule → ToolReport. Raw config text and the parsed
//! document are dropped before this function returns.
use crate::discovery::DiscoveryRoot;
use crate::engine::{ConfigState, evaluate_rule};
use crate::harness::HarnessId;
use crate::parse::{ParseFailure, extract_key, parse_config};
use crate::readfs::{ConfigReadOutcome, PathProbe, probe_directory, read_config};
use crate::version::{binary_on_path, detect_version};
use harness_guard_rules::loader::ValidatedRule;
use harness_guard_rules::report::{Confidence, ToolReport};
use std::collections::BTreeMap;

pub struct ScanResult {
    pub tool_report: ToolReport,
    /// true iff the scan degraded (unreadable/unparseable config) → exit 2.
    pub degraded: bool,
    /// Parse location/message only. Raw source text is never retained.
    pub parse_failure: Option<ParseFailure>,
}

/// Confidence in tool detection, shared by scan reports and detection-only
/// surfaces so the same evidence always receives the same label.
pub fn detection_confidence(
    detected_version: Option<&str>,
    codex_home_detected: bool,
) -> Confidence {
    match (detected_version, codex_home_detected) {
        (Some(_), true) => Confidence::High,
        (Some(_), false) | (None, true) => Confidence::Medium,
        (None, false) => Confidence::Low,
    }
}

/// Returns `None` iff neither the injected Codex home nor a regular Codex
/// entry in the injected path directories is present.
pub fn scan_codex(root: &DiscoveryRoot, rules: &[ValidatedRule]) -> Option<ScanResult> {
    let home_detected = probe_directory(&root.codex_home) != PathProbe::Missing;
    let on_path = binary_on_path(root, HarnessId::Codex);
    if !home_detected && !on_path {
        return None;
    }

    let detected_version = detect_version(root, HarnessId::Codex);
    let mut parse_failure = None;
    let mut config_paths = Vec::new();

    let config_state = match read_config(root, HarnessId::Codex) {
        ConfigReadOutcome::NoConfig => ConfigState::Missing,
        ConfigReadOutcome::Refused(reason) => {
            config_paths.push(
                root.config_path(HarnessId::Codex)
                    .to_string_lossy()
                    .into_owned(),
            );
            ConfigState::Unreadable(reason)
        }
        ConfigReadOutcome::Ok(text) => {
            config_paths.push(
                root.config_path(HarnessId::Codex)
                    .to_string_lossy()
                    .into_owned(),
            );
            match parse_config(&text) {
                Err(failure) => {
                    parse_failure = Some(failure.clone());
                    ConfigState::Unparseable(failure)
                }
                Ok(document) => {
                    let mut extracted = BTreeMap::new();
                    for rule in rules {
                        let key = rule.raw().observation.key.clone();
                        let value = extract_key(&document, &key);
                        extracted.insert(key, value);
                    }
                    // The parsed document and every unrelated value drop here.
                    ConfigState::Parsed(extracted)
                }
            }
        }
    };

    let degraded = matches!(
        &config_state,
        ConfigState::Unreadable(_) | ConfigState::Unparseable(_)
    );

    let mut findings: Vec<_> = rules
        .iter()
        .map(|rule| evaluate_rule(rule, &config_state, detected_version.as_deref()))
        .collect();
    findings.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));

    let version_in_range = detected_version
        .as_deref()
        .map(|version| {
            rules
                .iter()
                .all(|rule| crate::version::version_in_range(version, &rule.raw().tested_versions))
        })
        .unwrap_or(false);

    let (rules_last_verified_version, rules_verified_date) = rules
        .first()
        .map(|rule| {
            let tested = &rule.raw().tested_versions[0];
            (Some(tested.max.clone()), Some(tested.verified_on.clone()))
        })
        .unwrap_or((None, None));

    let detection_confidence = detection_confidence(detected_version.as_deref(), home_detected);

    Some(ScanResult {
        tool_report: ToolReport {
            tool: "codex".to_string(),
            detected_version,
            config_paths,
            detection_confidence,
            rules_last_verified_version,
            rules_verified_date,
            version_in_range,
            findings,
        },
        degraded,
        parse_failure,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use harness_guard_rules::loader::load_rules;
    use harness_guard_rules::report::Status;

    #[test]
    fn detection_confidence_matrix_is_explicit() {
        assert_eq!(
            detection_confidence(Some("0.144.5"), true),
            Confidence::High
        );
        assert_eq!(
            detection_confidence(Some("0.144.5"), false),
            Confidence::Medium
        );
        assert_eq!(detection_confidence(None, true), Confidence::Medium);
        assert_eq!(detection_confidence(None, false), Confidence::Low);
    }

    #[test]
    fn undetected_tool_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let root = DiscoveryRoot {
            codex_home: base.join("absent"),
            claude_home: base.join("absent-claude-home"),
            grok_home: base.join("absent-grok-home"),
            path_dirs: vec![],
        };
        assert!(scan_codex(&root, &load_rules()).is_none());
    }

    #[test]
    fn malformed_config_degrades_with_unknown_findings() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let home = base.join("codex-home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("config.toml"), "[history\n").unwrap();
        let root = DiscoveryRoot {
            codex_home: home,
            claude_home: base.join("absent-claude-home"),
            grok_home: base.join("absent-grok-home"),
            path_dirs: vec![],
        };
        let result = scan_codex(&root, &load_rules()).unwrap();
        assert!(result.degraded);
        assert!(result.parse_failure.is_some());
        assert!(
            result
                .tool_report
                .findings
                .iter()
                .all(|finding| finding.status == Status::Unknown)
        );
    }

    #[test]
    fn findings_are_sorted_by_rule_id() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let home = base.join("codex-home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("config.toml"), "").unwrap();
        let root = DiscoveryRoot {
            codex_home: home,
            claude_home: base.join("absent-claude-home"),
            grok_home: base.join("absent-grok-home"),
            path_dirs: vec![],
        };
        let result = scan_codex(&root, &load_rules()).unwrap();
        let ids: Vec<_> = result
            .tool_report
            .findings
            .iter()
            .map(|finding| finding.rule_id.clone())
            .collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }
}
