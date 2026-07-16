use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn compiled(schema_file: &str) -> jsonschema::Validator {
    let raw = std::fs::read_to_string(repo_root().join("schemas").join(schema_file)).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    jsonschema::validator_for(&json).unwrap()
}

#[test]
fn every_rule_file_validates_against_rule_schema() {
    let v = compiled("rule.schema.json");
    let rules_dir = repo_root().join("rules");
    let mut seen = 0;
    for entry in walk_json(&rules_dir) {
        if entry.file_name().unwrap() == "ruleset.json" {
            continue;
        }
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&entry).unwrap()).unwrap();
        assert!(
            v.validate(&json).is_ok(),
            "schema violation in {entry:?}: {:?}",
            v.iter_errors(&json)
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
        );
        seen += 1;
    }
    assert_eq!(seen, 1, "slice ships exactly one rule");
}

#[test]
fn rule_missing_source_fails_schema_validation() {
    // Negative test proving the structural citation constraint (§10.4).
    let v = compiled("rule.schema.json");
    let raw =
        std::fs::read_to_string(repo_root().join("rules/codex/history-persist-01.json")).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    json["sources"] = serde_json::json!([]);
    assert!(
        v.validate(&json).is_err(),
        "a rule with a non-unknown outcome and no sources MUST fail validation"
    );
}

#[test]
fn rule_schema_rejects_source_urls_without_https_hosts() {
    let validator = compiled("rule.schema.json");
    let mut json = rule_json();
    json["sources"][0]["url"] = serde_json::json!("https:///missing-host");
    assert!(validator.validate(&json).is_err());

    let mut json = rule_json();
    json["sources"][0]["archived_url"] = serde_json::json!("https:///missing-host");
    assert!(validator.validate(&json).is_err());
}

#[test]
fn finding_outcome_with_null_severity_fails_schema_validation() {
    // Regression pin for review finding 4: presence alone is insufficient.
    let v = compiled("rule.schema.json");
    let raw =
        std::fs::read_to_string(repo_root().join("rules/codex/history-persist-01.json")).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let finding = json["outcomes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|outcome| outcome["status"] == "finding")
        .unwrap();
    finding["severity"] = serde_json::Value::Null;
    assert!(
        v.validate(&json).is_err(),
        "a finding outcome with null severity MUST fail validation"
    );
}

#[test]
fn finding_outcome_with_null_confidence_fails_schema_validation() {
    let v = compiled("rule.schema.json");
    let mut json = rule_json();
    let finding = outcome_json_mut(&mut json, "finding");
    finding["confidence"] = serde_json::Value::Null;
    assert!(
        v.validate(&json).is_err(),
        "a finding outcome with null confidence MUST fail validation"
    );
}

#[test]
fn pass_outcome_with_null_confidence_fails_schema_validation() {
    let v = compiled("rule.schema.json");
    let mut json = rule_json();
    let pass = outcome_json_mut(&mut json, "pass");
    pass["confidence"] = serde_json::Value::Null;
    assert!(
        v.validate(&json).is_err(),
        "a pass outcome with null confidence MUST fail validation"
    );
}

#[test]
fn rule_outcome_schema_rejects_cross_status_fields() {
    let v = compiled("rule.schema.json");
    let cases = [
        (
            "pass",
            "remediation",
            serde_json::json!({
                "summary": "Not valid for a pass outcome.",
                "command": "synthetic command"
            }),
        ),
        (
            "pass",
            "unknown_reason",
            serde_json::json!("Not valid for a pass outcome."),
        ),
        (
            "pass",
            "verify_url",
            serde_json::json!("https://example.invalid/verify"),
        ),
        (
            "finding",
            "unknown_reason",
            serde_json::json!("Not valid for a finding outcome."),
        ),
        (
            "finding",
            "verify_url",
            serde_json::json!("https://example.invalid/verify"),
        ),
        ("unknown", "severity", serde_json::json!("info")),
        ("unknown", "confidence", serde_json::json!("low")),
        (
            "unknown",
            "remediation",
            serde_json::json!({
                "summary": "Not valid for an unknown outcome.",
                "command": "synthetic command"
            }),
        ),
    ];

    for (status, field, value) in cases {
        let mut json = rule_json();
        outcome_json_mut(&mut json, status)[field] = value;
        assert!(
            v.validate(&json).is_err(),
            "{field} must be rejected for rule outcome status {status}"
        );
    }
}

#[test]
fn report_finding_with_null_severity_fails_schema_validation() {
    let v = compiled("report.schema.json");
    let mut json = valid_report_with_status("finding", serde_json::json!("warning"));
    assert!(
        v.validate(&json).is_ok(),
        "the synthetic finding baseline must validate before mutation"
    );
    json["tools"][0]["findings"][0]["severity"] = serde_json::Value::Null;
    assert!(
        v.validate(&json).is_err(),
        "a report finding with null severity MUST fail validation"
    );
}

#[test]
fn report_pass_with_warning_severity_fails_schema_validation() {
    let v = compiled("report.schema.json");
    let mut json = valid_report_with_status("pass", serde_json::Value::Null);
    assert!(
        v.validate(&json).is_ok(),
        "the synthetic pass baseline must validate before mutation"
    );
    json["tools"][0]["findings"][0]["severity"] = serde_json::json!("warning");
    assert!(
        v.validate(&json).is_err(),
        "a report pass with warning severity MUST fail validation"
    );
}

#[test]
fn report_schema_rejects_cross_status_fields() {
    let v = compiled("report.schema.json");
    let cases = [
        (
            "pass",
            "remediation",
            serde_json::json!({
                "summary": "Not valid for a pass result.",
                "command": "synthetic command"
            }),
        ),
        (
            "pass",
            "unknown_reason",
            serde_json::json!("Not valid for a pass result."),
        ),
        (
            "pass",
            "stale_reason",
            serde_json::json!("Not valid for a pass result."),
        ),
        (
            "finding",
            "unknown_reason",
            serde_json::json!("Not valid for a finding result."),
        ),
        (
            "finding",
            "stale_reason",
            serde_json::json!("Not valid for a finding result."),
        ),
        ("unknown", "severity", serde_json::json!("info")),
        ("unknown", "confidence", serde_json::json!("low")),
        (
            "unknown",
            "source",
            serde_json::json!({
                "url": "https://example.invalid/source",
                "retrieved": "2026-07-16"
            }),
        ),
        (
            "unknown",
            "remediation",
            serde_json::json!({
                "summary": "Not valid for an unknown result.",
                "command": "synthetic command"
            }),
        ),
        (
            "unknown",
            "stale_reason",
            serde_json::json!("Not valid for an unknown result."),
        ),
        ("unknown", "unknown_reason", serde_json::json!("")),
        ("stale-ruleset", "severity", serde_json::json!("info")),
        ("stale-ruleset", "confidence", serde_json::json!("low")),
        ("stale-ruleset", "source", serde_json::Value::Null),
        (
            "stale-ruleset",
            "remediation",
            serde_json::json!({
                "summary": "Not valid for a stale result.",
                "command": "synthetic command"
            }),
        ),
        (
            "stale-ruleset",
            "unknown_reason",
            serde_json::json!("Not valid for a stale result."),
        ),
        ("stale-ruleset", "stale_reason", serde_json::json!("")),
    ];

    for (status, field, value) in cases {
        let mut json = valid_report_with_exact_status(status);
        assert!(
            v.validate(&json).is_ok(),
            "synthetic {status} baseline must validate before mutation: {:?}",
            v.iter_errors(&json)
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
        );
        json["tools"][0]["findings"][0][field] = value;
        assert!(
            v.validate(&json).is_err(),
            "{field} must be rejected for report status {status}"
        );
    }
}

#[test]
fn report_schema_rejects_malformed_source_url_and_date() {
    let validator = compiled("report.schema.json");
    let mut malformed_url = valid_report_with_exact_status("pass");
    malformed_url["tools"][0]["findings"][0]["source"]["url"] =
        serde_json::json!("https:///missing-host");
    assert!(validator.validate(&malformed_url).is_err());

    let mut malformed_date = valid_report_with_exact_status("pass");
    malformed_date["tools"][0]["findings"][0]["source"]["retrieved"] =
        serde_json::json!("2026/07/16");
    assert!(validator.validate(&malformed_date).is_err());
}

#[test]
fn embedded_source_def_matches_source_schema() {
    // rule.schema.json embeds a copy of source.schema.json in $defs — pin them equal.
    let rule: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("schemas/rule.schema.json")).unwrap(),
    )
    .unwrap();
    let source: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("schemas/source.schema.json")).unwrap(),
    )
    .unwrap();
    let embedded = &rule["$defs"]["source"];
    for k in ["type", "required", "additionalProperties", "properties"] {
        assert_eq!(
            embedded[k], source[k],
            "drift between rule.schema.json $defs/source and source.schema.json at `{k}`"
        );
    }
}

#[test]
fn rules_load_and_validate_via_types() {
    let rules = harness_guard_rules::loader::load_rules();
    assert_eq!(rules.len(), 1);
    let r = &rules[0];
    assert_eq!(r.raw().id, "codex-history-persist-01");
    assert_eq!(
        r.raw().observation.allowed_render,
        vec!["save-all", "none", "unset"]
    );
    assert!(r.primary_source().url.starts_with("https://"));
    assert!(!r.primary_source().retrieved.is_empty());
    assert!(!r.raw().limitations.is_empty());
    assert!(!r.raw().unknown_conditions.is_empty());
}

#[test]
fn ruleset_version_is_calver() {
    let v = harness_guard_rules::loader::ruleset_version();
    let parts: Vec<&str> = v.split('.').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].len(), 4);
}

#[test]
fn non_unknown_outcome_without_source_is_unconstructible() {
    let rules = harness_guard_rules::loader::load_rules();
    let mut raw = rules[0].raw().clone();
    raw.sources.clear();
    assert!(harness_guard_rules::loader::ValidatedRule::try_from_raw(raw).is_err());
}

#[test]
fn unknown_outcome_status_is_rejected_by_runtime_validation() {
    assert_raw_rejected(|raw| raw.outcomes[0].status = "surprise".into());
}

#[test]
fn invalid_applicability_metadata_is_rejected_by_runtime_validation() {
    assert_raw_rejected(|raw| raw.category = "bogus".into());
    assert_raw_rejected(|raw| raw.os.clear());
    assert_raw_rejected(|raw| raw.os = vec!["bogus".into()]);
    assert_raw_rejected(|raw| raw.scopes.clear());
    assert_raw_rejected(|raw| raw.scopes = vec!["bogus".into()]);
}

#[test]
fn pass_outcome_matrix_is_enforced_by_runtime_validation() {
    assert_raw_rejected(|raw| outcome_mut(raw, "pass").confidence = None);
    assert_raw_rejected(|raw| outcome_mut(raw, "pass").confidence = Some("certain".into()));
    assert_raw_rejected(|raw| outcome_mut(raw, "pass").severity = Some("info".into()));
    assert_raw_rejected(|raw| {
        outcome_mut(raw, "pass").remediation = Some(harness_guard_rules::schema::Remediation {
            summary: "not valid for pass".into(),
            command: "none".into(),
        });
    });
}

#[test]
fn finding_outcome_matrix_is_enforced_by_runtime_validation() {
    assert_raw_rejected(|raw| outcome_mut(raw, "finding").severity = None);
    assert_raw_rejected(|raw| outcome_mut(raw, "finding").severity = Some("critical".into()));
    assert_raw_rejected(|raw| outcome_mut(raw, "finding").confidence = None);
    assert_raw_rejected(|raw| outcome_mut(raw, "finding").confidence = Some("certain".into()));
    assert_raw_rejected(|raw| {
        outcome_mut(raw, "finding")
            .remediation
            .as_mut()
            .unwrap()
            .command
            .clear();
    });
}

#[test]
fn unknown_outcome_matrix_is_enforced_by_runtime_validation() {
    assert_raw_rejected(|raw| outcome_mut(raw, "unknown").severity = Some("info".into()));
    assert_raw_rejected(|raw| outcome_mut(raw, "unknown").confidence = Some("low".into()));
    assert_raw_rejected(|raw| outcome_mut(raw, "unknown").unknown_reason = None);
    assert_raw_rejected(|raw| outcome_mut(raw, "unknown").unknown_reason = Some(String::new()));
    assert_raw_rejected(|raw| {
        outcome_mut(raw, "unknown").remediation = Some(harness_guard_rules::schema::Remediation {
            summary: "not valid for unknown".into(),
            command: "none".into(),
        });
    });
}

#[test]
fn malformed_sources_are_rejected_by_runtime_validation() {
    assert_raw_rejected(|raw| raw.sources[0].schema_version = "2.0".into());
    assert_raw_rejected(|raw| raw.sources[0].url = "http://example.invalid".into());
    assert_raw_rejected(|raw| raw.sources[0].url = "https:///missing-host".into());
    assert_raw_rejected(|raw| raw.sources[0].publisher.clear());
    assert_raw_rejected(|raw| raw.sources[0].title.clear());
    assert_raw_rejected(|raw| raw.sources[0].evidence_class = "rumor".into());
    assert_raw_rejected(|raw| raw.sources[0].retrieved = "2026-02-30".into());
    assert_raw_rejected(|raw| raw.sources[0].content_hash = "sha256:abcd".into());
    assert_raw_rejected(|raw| {
        raw.sources[0].archived_url = Some("http://archive.example.invalid".into());
    });
}

#[test]
fn malformed_tested_versions_are_rejected_by_runtime_validation() {
    assert_raw_rejected(|raw| raw.tested_versions[0].min = "v0.144.5".into());
    assert_raw_rejected(|raw| raw.tested_versions[0].max = "0.144".into());
    assert_raw_rejected(|raw| raw.tested_versions[0].verified_on = "2026-02-30".into());
    assert_raw_rejected(|raw| {
        raw.tested_versions[0].min = "0.200.0".into();
        raw.tested_versions[0].max = "0.144.5".into();
    });
    assert_raw_rejected(|raw| raw.tested_versions[0].min = "<=0.100.0".into());
}

fn rule_json() -> serde_json::Value {
    let raw =
        std::fs::read_to_string(repo_root().join("rules/codex/history-persist-01.json")).unwrap();
    serde_json::from_str(&raw).unwrap()
}

fn outcome_json_mut<'a>(
    json: &'a mut serde_json::Value,
    status: &str,
) -> &'a mut serde_json::Value {
    json["outcomes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|outcome| outcome["status"] == status)
        .unwrap()
}

fn raw_rule() -> harness_guard_rules::schema::RawRule {
    harness_guard_rules::loader::load_rules()[0].raw().clone()
}

fn outcome_mut<'a>(
    raw: &'a mut harness_guard_rules::schema::RawRule,
    status: &str,
) -> &'a mut harness_guard_rules::schema::RawOutcome {
    raw.outcomes
        .iter_mut()
        .find(|outcome| outcome.status == status)
        .unwrap()
}

fn assert_raw_rejected(mutate: impl FnOnce(&mut harness_guard_rules::schema::RawRule)) {
    let mut raw = raw_rule();
    mutate(&mut raw);
    assert!(
        harness_guard_rules::loader::ValidatedRule::try_from_raw(raw).is_err(),
        "mutated raw rule must fail runtime validation"
    );
}

fn valid_report_with_status(status: &str, severity: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "1.0",
        "harness_guard_version": "0.1.0",
        "ruleset_version": "2026.07.15",
        "scanned_at": "2026-07-16T00:00:00Z",
        "network_requests_made": 0,
        "platform": { "os": "linux" },
        "tools": [{
            "tool": "codex",
            "detected_version": "0.144.5",
            "config_paths": ["~/.codex/config.toml"],
            "detection_confidence": "high",
            "rules_last_verified_version": "0.144.5",
            "rules_verified_date": "2026-07-16",
            "version_in_range": true,
            "findings": [{
                "rule_id": "codex-history-persist-01",
                "status": status,
                "severity": severity,
                "confidence": "high",
                "evidence_class": "official-documentation",
                "message": "Synthetic report schema test.",
                "observation": "history.persistence = allowlisted-value",
                "remediation": null,
                "source": {
                    "url": "https://example.invalid/official-documentation",
                    "retrieved": "2026-07-16"
                },
                "valid_from": null,
                "valid_until": "0.144.5",
                "limitations": ["Synthetic test limitation."],
                "unknown_reason": null,
                "verify_url": null,
                "stale_reason": null
            }]
        }],
        "summary": {
            "tools_scanned": 1,
            "warning": 1,
            "info": 0,
            "unknown": 0,
            "stale": 0,
            "passed": 0
        }
    })
}

fn valid_report_with_exact_status(status: &str) -> serde_json::Value {
    let mut json = match status {
        "pass" => valid_report_with_status("pass", serde_json::Value::Null),
        "finding" => valid_report_with_status("finding", serde_json::json!("warning")),
        "unknown" => {
            let mut json = valid_report_with_status("unknown", serde_json::Value::Null);
            let finding = &mut json["tools"][0]["findings"][0];
            finding["confidence"] = serde_json::Value::Null;
            finding["source"] = serde_json::Value::Null;
            finding["unknown_reason"] = serde_json::json!("Synthetic unknown reason.");
            json
        }
        "stale-ruleset" => {
            let mut json = valid_report_with_status("stale-ruleset", serde_json::Value::Null);
            let finding = &mut json["tools"][0]["findings"][0];
            finding["confidence"] = serde_json::Value::Null;
            finding["stale_reason"] = serde_json::json!("Synthetic stale reason.");
            json
        }
        other => panic!("unsupported synthetic status {other}"),
    };
    json["tools"][0]["findings"][0]["status"] = serde_json::json!(status);
    json
}

fn walk_json(dir: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(walk_json(&p));
        } else if p.extension().is_some_and(|x| x == "json") {
            out.push(p);
        }
    }
    out.sort();
    out
}
