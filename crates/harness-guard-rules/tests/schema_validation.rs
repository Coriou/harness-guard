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
    assert_eq!(r.raw.id, "codex-history-persist-01");
    assert_eq!(
        r.raw.observation.allowed_render,
        vec!["save-all", "none", "unset"]
    );
    assert!(r.primary_source.url.starts_with("https://"));
    assert!(!r.primary_source.retrieved.is_empty());
    assert!(!r.raw.limitations.is_empty());
    assert!(!r.raw.unknown_conditions.is_empty());
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
    let mut raw = rules[0].raw.clone();
    raw.sources.clear();
    assert!(harness_guard_rules::loader::ValidatedRule::try_from_raw(raw).is_err());
}

fn valid_report_with_status(status: &str, severity: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "1.0",
        "harness_guard_version": "0.1.0",
        "ruleset_version": "2026.07.15",
        "scanned_at": "2026-07-15T00:00:00Z",
        "network_requests_made": 0,
        "platform": { "os": "linux" },
        "tools": [{
            "tool": "codex",
            "detected_version": "0.144.4",
            "config_paths": ["~/.codex/config.toml"],
            "detection_confidence": "high",
            "rules_last_verified_version": "0.144.4",
            "rules_verified_date": "2026-07-15",
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
                    "retrieved": "2026-07-15"
                },
                "valid_from": null,
                "valid_until": "0.144.4",
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
