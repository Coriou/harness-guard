//! The scan orchestrator: discovery → bounded read → parse → extract →
//! evaluate each rule → ToolReport. Raw config text and the parsed
//! document are dropped before this function returns.
use crate::discovery::DiscoveryRoot;
use crate::engine::{ConfigState, evaluate_rule};
use crate::harness::{ConfigFormat, HarnessId, descriptor};
use crate::parse::{ParseFailure, extract_key, parse_config};
use crate::parse_json::{extract_key_json, parse_config_json};
use crate::readfs::{ConfigReadOutcome, PathProbe, probe_directory, read_config};
use crate::version::{binary_on_path, detect_version, parse_version};
use harness_guard_rules::loader::ValidatedRule;
use harness_guard_rules::report::{Confidence, ToolReport};
use harness_guard_rules::schema::TestedVersion;
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

/// §5.5 (decision j): rules_last_verified_version is the MINIMUM of the
/// rules' greatest tested maxes (the weakest guarantee) and
/// rules_verified_date the EARLIEST verified_on among those greatest-max
/// entries — conservative in both dimensions. Fixes the former
/// rules.first() shortcut before >1 rule can mislead.
pub fn conservative_aggregates(rules: &[&ValidatedRule]) -> (Option<String>, Option<String>) {
    let mut greatest_per_rule: Vec<&TestedVersion> = Vec::new();
    for rule in rules {
        let greatest = rule
            .raw()
            .tested_versions
            .iter()
            .max_by_key(|tested| parse_version(&tested.max).unwrap_or((0, 0, 0)));
        match greatest {
            Some(tested) => greatest_per_rule.push(tested),
            None => return (None, None),
        }
    }
    let weakest_version = greatest_per_rule
        .iter()
        .min_by_key(|tested| parse_version(&tested.max).unwrap_or((0, 0, 0)))
        .map(|tested| tested.max.clone());
    // ISO dates: lexicographic order IS chronological order.
    let earliest_date = greatest_per_rule
        .iter()
        .map(|tested| tested.verified_on.as_str())
        .min()
        .map(str::to_string);
    (weakest_version, earliest_date)
}

enum ParsedDocument {
    Toml(toml::Value),
    Json(serde_json::Value),
}

/// Returns None iff neither the harness home nor a PATH marker is present
/// (§5.5). Rules are filtered to this harness's tool id.
pub fn scan_harness(
    root: &DiscoveryRoot,
    harness: HarnessId,
    rules: &[ValidatedRule],
) -> Option<ScanResult> {
    let facts = descriptor(harness);
    let home_detected = probe_directory(root.home(harness)) != PathProbe::Missing;
    let on_path = binary_on_path(root, harness);
    if !home_detected && !on_path {
        return None;
    }

    let harness_rules: Vec<&ValidatedRule> = rules
        .iter()
        .filter(|rule| rule.raw().tool == harness.as_str())
        .collect();

    let detected_version = detect_version(root, harness);
    let mut parse_failure = None;
    let mut config_paths = Vec::new();

    let config_state = match read_config(root, harness) {
        ConfigReadOutcome::NoConfig => ConfigState::Missing,
        ConfigReadOutcome::Refused(reason) => {
            config_paths.push(root.config_path(harness).to_string_lossy().into_owned());
            ConfigState::Unreadable(reason)
        }
        ConfigReadOutcome::Ok(text) => {
            config_paths.push(root.config_path(harness).to_string_lossy().into_owned());
            let parsed = match facts.config_format {
                ConfigFormat::Toml => parse_config(&text).map(ParsedDocument::Toml),
                ConfigFormat::Json => parse_config_json(&text).map(ParsedDocument::Json),
            };
            match parsed {
                Err(failure) => {
                    parse_failure = Some(failure.clone());
                    ConfigState::Unparseable(failure)
                }
                Ok(document) => {
                    let mut extracted = BTreeMap::new();
                    for rule in &harness_rules {
                        let key = rule.raw().observation.key.clone();
                        let value = match &document {
                            ParsedDocument::Toml(doc) => extract_key(doc, &key),
                            ParsedDocument::Json(doc) => extract_key_json(doc, &key),
                        };
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

    let mut findings: Vec<_> = harness_rules
        .iter()
        .map(|rule| evaluate_rule(rule, &config_state, detected_version.as_deref()))
        .collect();
    findings.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));

    let version_in_range = detected_version
        .as_deref()
        .map(|version| {
            harness_rules
                .iter()
                .all(|rule| crate::version::version_in_range(version, &rule.raw().tested_versions))
        })
        .unwrap_or(false);

    let (rules_last_verified_version, rules_verified_date) =
        conservative_aggregates(&harness_rules);
    let detection_confidence = detection_confidence(detected_version.as_deref(), home_detected);

    Some(ScanResult {
        tool_report: ToolReport {
            tool: harness.as_str().to_string(),
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
        assert!(scan_harness(&root, HarnessId::Codex, &load_rules()).is_none());
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
        let result = scan_harness(&root, HarnessId::Codex, &load_rules()).unwrap();
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
    fn per_tool_aggregates_are_conservative_in_both_dimensions() {
        // Two synthetic rules: greatest maxes 0.144.5 (verified 2026-07-16) and
        // 0.150.0 (verified 2026-07-10). Weakest guarantee: min of maxes =
        // 0.144.5; earliest date = 2026-07-10.
        let rules = load_rules();
        let mut newer = rules[0].raw().clone();
        newer.id = "codex-synthetic-02".to_string();
        newer.tested_versions = vec![harness_guard_rules::schema::TestedVersion {
            min: "<=0.150.0".to_string(),
            max: "0.150.0".to_string(),
            verified_on: "2026-07-10".to_string(),
        }];
        let newer = harness_guard_rules::loader::ValidatedRule::try_from_raw(newer).unwrap();
        let pair = [&rules[0], &newer];
        let (version, date) = conservative_aggregates(&pair);
        assert_eq!(version.as_deref(), Some("0.144.5"));
        assert_eq!(date.as_deref(), Some("2026-07-10"));
        assert_eq!(conservative_aggregates(&[]), (None, None));
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
        let result = scan_harness(&root, HarnessId::Codex, &load_rules()).unwrap();
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
