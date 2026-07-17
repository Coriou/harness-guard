use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const CASES: [&str; 13] = [
    "deep-nesting",
    "hardened",
    "malformed-toml",
    "minimal",
    "missing",
    "oversized",
    "permission-denied",
    "risky-explicit",
    "risky-unset",
    "symlink-config",
    "unknown-version",
    "unrecognized-value",
    "version-out-of-range",
];

const IN_RANGE_VERSION_CASES: [&str; 10] = [
    "deep-nesting",
    "hardened",
    "malformed-toml",
    "minimal",
    "oversized",
    "permission-denied",
    "risky-explicit",
    "risky-unset",
    "symlink-config",
    "unrecognized-value",
];

/// claude-code fixture matrix (Task 18): the 13-case codex pattern (with
/// malformed-toml renamed malformed-json for the JSON format) plus 5
/// JSON-specific hostile additions.
const CLAUDE_CASES: [&str; 18] = [
    "deep-nesting",
    "duplicate-keys",
    "float-where-integer",
    "hardened",
    "huge-number",
    "malformed-json",
    "minimal",
    "missing",
    "non-utf8",
    "oversized",
    "permission-denied",
    "risky-explicit",
    "risky-unset",
    "secret-shaped",
    "symlink-config",
    "unknown-version",
    "unrecognized-value",
    "version-out-of-range",
];

/// Every CLAUDE_CASES entry except missing/unknown-version/version-out-of-range
/// (mirrors IN_RANGE_VERSION_CASES above).
const CLAUDE_IN_RANGE_VERSION_CASES: [&str; 15] = [
    "deep-nesting",
    "duplicate-keys",
    "float-where-integer",
    "hardened",
    "huge-number",
    "malformed-json",
    "minimal",
    "non-utf8",
    "oversized",
    "permission-denied",
    "risky-explicit",
    "risky-unset",
    "secret-shaped",
    "symlink-config",
    "unrecognized-value",
];

/// grok-build fixture matrix (Task 19): 13-case TOML pattern as codex, plus
/// managed-install-version (symlink basename detection; PATH symlink is
/// runtime-only like symlink-config).
const GROK_CASES: [&str; 14] = [
    "deep-nesting",
    "hardened",
    "malformed-toml",
    "managed-install-version",
    "minimal",
    "missing",
    "oversized",
    "permission-denied",
    "risky-explicit",
    "risky-unset",
    "symlink-config",
    "unknown-version",
    "unrecognized-value",
    "version-out-of-range",
];

/// npm-layout in-range cases (committed package.json @ 0.2.102). Managed-install
/// is asserted separately — no package.json; version comes from the runtime
/// PATH symlink target basename.
const GROK_IN_RANGE_VERSION_CASES: [&str; 10] = [
    "deep-nesting",
    "hardened",
    "malformed-toml",
    "minimal",
    "oversized",
    "permission-denied",
    "risky-explicit",
    "risky-unset",
    "symlink-config",
    "unrecognized-value",
];

/// Mixed multi-harness fixtures (§11.2 aggregation); the per-harness CASES
/// arrays above stay per-harness.
const MIXED_CASES: [&str; 1] = ["codex-pass-claude-degraded"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn fixtures_root() -> PathBuf {
    repo_root().join("fixtures").canonicalize().unwrap()
}

fn expected(case: &str) -> serde_json::Value {
    expected_for("codex", case)
}

fn expected_for(tool: &str, case: &str) -> serde_json::Value {
    let path = fixtures_root().join(tool).join(case).join("expected.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn every_expected_json_validates_against_fixture_schema() {
    let schema_raw =
        std::fs::read_to_string(repo_root().join("schemas/fixture.schema.json")).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_raw).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();

    let case_dirs = std::fs::read_dir(fixtures_root().join("codex"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>();
    let expected_cases = CASES
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        case_dirs, expected_cases,
        "the fixture matrix must contain exactly the 13 named cases"
    );

    for case in CASES {
        let path = fixtures_root()
            .join("codex")
            .join(case)
            .join("expected.json");
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(
            validator.validate(&json).is_ok(),
            "schema violation in {path:?}: {:?}",
            validator
                .iter_errors(&json)
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(json["case"], case, "case field must equal directory name");
        assert_eq!(
            json["expected_report"]["network_requests_made"], 0,
            "{case} must pin the no-egress report field"
        );
    }
}

#[test]
fn every_claude_code_expected_json_validates_against_fixture_schema() {
    let schema_raw =
        std::fs::read_to_string(repo_root().join("schemas/fixture.schema.json")).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_raw).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();

    let case_dirs = std::fs::read_dir(fixtures_root().join("claude-code"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>();
    let expected_cases = CLAUDE_CASES
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        case_dirs, expected_cases,
        "the claude-code fixture matrix must contain exactly the 18 named cases"
    );

    for case in CLAUDE_CASES {
        let path = fixtures_root()
            .join("claude-code")
            .join(case)
            .join("expected.json");
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(
            validator.validate(&json).is_ok(),
            "schema violation in {path:?}: {:?}",
            validator
                .iter_errors(&json)
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(json["case"], case, "case field must equal directory name");
        assert_eq!(
            json["expected_report"]["network_requests_made"], 0,
            "{case} must pin the no-egress report field"
        );
    }
}

#[test]
fn every_mixed_expected_json_validates_against_fixture_schema() {
    let schema_raw =
        std::fs::read_to_string(repo_root().join("schemas/fixture.schema.json")).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_raw).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();

    let case_dirs = std::fs::read_dir(fixtures_root().join("mixed"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>();
    let expected_cases = MIXED_CASES
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        case_dirs, expected_cases,
        "the mixed fixture directory must contain exactly the named cases"
    );

    for case in MIXED_CASES {
        let path = fixtures_root()
            .join("mixed")
            .join(case)
            .join("expected.json");
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(
            validator.validate(&json).is_ok(),
            "schema violation in {path:?}: {:?}",
            validator
                .iter_errors(&json)
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(json["case"], case, "case field must equal directory name");
        assert_eq!(
            json["expected_report"]["network_requests_made"], 0,
            "{case} must pin the no-egress report field"
        );
        let tools = json["expected_report"]["tools"].as_array().unwrap();
        assert_eq!(
            tools.len(),
            2,
            "the mixed fixture must aggregate exactly two tool entries"
        );
        assert_eq!(tools[0]["tool"], "claude-code");
        assert_eq!(tools[1]["tool"], "codex");
    }
}

#[test]
fn in_range_fixture_version_markers_are_synthetic_and_exact_latest() {
    let codex_root = fixtures_root().join("codex");
    for case in IN_RANGE_VERSION_CASES {
        let path = codex_root.join(case).join("files/path");
        assert_eq!(
            std::fs::read_to_string(path.join("codex")).unwrap(),
            "#!/usr/bin/env node\n// synthetic fixture shim — never executed by harness-guard\n"
        );
        let package: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path.join("package.json")).unwrap())
                .unwrap();
        assert_eq!(package["name"], "@openai/codex");
        assert_eq!(package["version"], "0.144.5");
    }

    let unknown_path = codex_root.join("unknown-version/files/path");
    assert!(unknown_path.join("codex").is_file());
    assert!(!unknown_path.join("package.json").exists());

    let out_of_range: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(codex_root.join("version-out-of-range/files/path/package.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(out_of_range["name"], "@openai/codex");
    assert_eq!(out_of_range["version"], "9.9.9");
}

#[test]
fn claude_code_in_range_fixture_version_markers_are_synthetic_and_exact_latest() {
    let claude_root = fixtures_root().join("claude-code");
    for case in CLAUDE_IN_RANGE_VERSION_CASES {
        let path = claude_root.join(case).join("files/path");
        assert_eq!(
            std::fs::read_to_string(path.join("claude")).unwrap(),
            "#!/usr/bin/env node\n// synthetic fixture shim — never executed by harness-guard\n"
        );
        let package: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path.join("package.json")).unwrap())
                .unwrap();
        assert_eq!(package["name"], "@anthropic-ai/claude-code");
        assert_eq!(package["version"], "2.1.204");
    }

    let unknown_path = claude_root.join("unknown-version/files/path");
    assert!(unknown_path.join("claude").is_file());
    assert!(!unknown_path.join("package.json").exists());

    let out_of_range: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(claude_root.join("version-out-of-range/files/path/package.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(out_of_range["name"], "@anthropic-ai/claude-code");
    assert_eq!(out_of_range["version"], "9.9.9");
}

#[test]
fn every_grok_build_expected_json_validates_against_fixture_schema() {
    let schema_raw =
        std::fs::read_to_string(repo_root().join("schemas/fixture.schema.json")).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_raw).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();

    let case_dirs = std::fs::read_dir(fixtures_root().join("grok-build"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>();
    let expected_cases = GROK_CASES
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        case_dirs, expected_cases,
        "the grok-build fixture matrix must contain exactly the 14 named cases"
    );

    for case in GROK_CASES {
        let path = fixtures_root()
            .join("grok-build")
            .join(case)
            .join("expected.json");
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(
            validator.validate(&json).is_ok(),
            "schema violation in {path:?}: {:?}",
            validator
                .iter_errors(&json)
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(json["case"], case, "case field must equal directory name");
        assert_eq!(
            json["expected_report"]["network_requests_made"], 0,
            "{case} must pin the no-egress report field"
        );
    }
}

#[test]
fn grok_build_in_range_fixture_version_markers_are_synthetic_and_exact_latest() {
    let grok_root = fixtures_root().join("grok-build");
    for case in GROK_IN_RANGE_VERSION_CASES {
        let path = grok_root.join(case).join("files/path");
        assert_eq!(
            std::fs::read_to_string(path.join("grok")).unwrap(),
            "#!/usr/bin/env node\n// synthetic fixture shim — never executed by harness-guard\n"
        );
        let package: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path.join("package.json")).unwrap())
                .unwrap();
        assert_eq!(package["name"], "@xai-official/grok");
        assert_eq!(package["version"], "0.2.102");
    }

    let unknown_path = grok_root.join("unknown-version/files/path");
    assert!(unknown_path.join("grok").is_file());
    assert!(!unknown_path.join("package.json").exists());

    // Managed-install: versioned binary target is committed under
    // path/downloads/; PATH `grok` symlink is created at test runtime
    // (hostile.rs) so fixture trees stay symlink-free on disk.
    let managed = grok_root.join("managed-install-version/files");
    assert!(!managed.join("path/package.json").exists());
    assert!(!managed.join("path/grok").exists());
    assert_eq!(
        std::fs::read_to_string(managed.join("path/downloads/grok-0.2.102-macos-x86_64")).unwrap(),
        "synthetic managed-install binary; never executed by harness-guard\n"
    );
    let managed_expected: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(grok_root.join("managed-install-version/expected.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        managed_expected["expected_report"]["tools"][0]["detected_version"],
        "0.2.102"
    );
    assert_eq!(
        managed_expected["expected_report"]["tools"][0]["version_in_range"],
        true
    );

    let out_of_range: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(grok_root.join("version-out-of-range/files/path/package.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(out_of_range["name"], "@xai-official/grok");
    assert_eq!(out_of_range["version"], "9.9.9");
}

#[test]
fn fixtures_contain_no_real_machine_leakage() {
    let mut stack = vec![fixtures_root()];
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            assert!(
                !metadata.file_type().is_symlink(),
                "committed symlink at {path:?}; hostile symlinks are runtime-only"
            );
            if metadata.is_dir() {
                stack.push(path);
                continue;
            }
            assert!(
                metadata.len() < 64 * 1024,
                "oversized committed fixture {path:?}"
            );
            let bytes = std::fs::read(&path).unwrap();
            if let Ok(text) = String::from_utf8(bytes) {
                for needle in [
                    "/Users/",
                    "/home/",
                    "C:\\Users",
                    "CODEX_HOME=",
                    "sk-",
                    "api_key",
                    "access_token",
                    "secret_key",
                ] {
                    assert!(
                        !text.contains(needle),
                        "fixture {path:?} contains forbidden fragment {needle:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn fixture_inputs_never_model_sensitive_data_stores() {
    let forbidden_names = [
        ".env",
        ".bash_history",
        ".zsh_history",
        "history.jsonl",
        "sessions",
        "shell-history",
        "transcripts",
    ];
    let mut stack = vec![
        fixtures_root().join("codex"),
        fixtures_root().join("claude-code"),
        fixtures_root().join("mixed"),
    ];
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path.clone());
            }
            let name = path.file_name().unwrap().to_string_lossy();
            assert!(
                !forbidden_names.contains(&name.as_ref()),
                "sensitive-data fixture path is forbidden: {path:?}"
            );
        }
    }
}

#[test]
fn hostile_runtime_bases_pin_structural_refusal_reasons() {
    for (case, reason) in [
        (
            "symlink-config",
            "config path contains a symlink or reparse point — not followed",
        ),
        ("oversized", "config file exceeds the 1 MiB parse bound"),
        (
            "permission-denied",
            "config file is not readable (permission denied)",
        ),
    ] {
        let json = expected(case);
        let finding = &json["expected_report"]["tools"][0]["findings"][0];
        assert_eq!(finding["status"], "unknown");
        assert_eq!(finding["unknown_reason"], reason);
        assert_eq!(finding["observation"], serde_json::Value::Null);
        assert_eq!(finding["source"], serde_json::Value::Null);
    }
}

#[test]
fn claude_code_hostile_runtime_bases_pin_structural_refusal_reasons() {
    for (case, reason) in [
        (
            "symlink-config",
            "config path contains a symlink or reparse point — not followed",
        ),
        ("oversized", "config file exceeds the 1 MiB parse bound"),
        (
            "permission-denied",
            "config file is not readable (permission denied)",
        ),
        ("non-utf8", "config file is not valid UTF-8"),
    ] {
        let json = expected_for("claude-code", case);
        for finding in json["expected_report"]["tools"][0]["findings"]
            .as_array()
            .unwrap()
        {
            assert_eq!(finding["status"], "unknown");
            assert_eq!(finding["unknown_reason"], reason);
            assert_eq!(finding["observation"], serde_json::Value::Null);
            assert_eq!(finding["source"], serde_json::Value::Null);
        }
    }
}

#[test]
fn unknown_version_pins_non_disclosing_fallback_for_unrecognized_value() {
    let case_root = fixtures_root().join("codex/unknown-version");
    let input = std::fs::read_to_string(case_root.join("files/codex-home/config.toml")).unwrap();
    assert!(
        input.contains("persistence = \"archive\""),
        "the fixture must exercise the unrecognized-value stale branch"
    );

    let expected_text = std::fs::read_to_string(case_root.join("expected.json")).unwrap();
    assert!(
        !expected_text.contains("archive"),
        "the hostile raw value must never enter expected output"
    );
    let json: serde_json::Value = serde_json::from_str(&expected_text).unwrap();
    let tool = &json["expected_report"]["tools"][0];
    assert_eq!(tool["detected_version"], serde_json::Value::Null);
    assert_eq!(tool["version_in_range"], false);
    // Findings are sorted by rule_id (Task 17 added rules that sort before
    // codex-history-persist-01), so locate this rule's finding by id rather
    // than assuming it is first.
    let finding = tool["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["rule_id"] == "codex-history-persist-01")
        .expect("codex-history-persist-01 finding must be present");
    assert_eq!(finding["status"], "stale-ruleset");
    assert_eq!(
        finding["message"],
        "Unverified — last-known rule indicates the configured value cannot be interpreted safely. Observed: unrecognized value (raw value withheld)."
    );
    assert_eq!(finding["observation"], serde_json::Value::Null);
}

#[test]
fn out_of_range_fixture_remains_safe_looking_but_stale() {
    let case_root = fixtures_root().join("codex/version-out-of-range");
    assert_eq!(
        std::fs::read_to_string(case_root.join("files/codex-home/config.toml")).unwrap(),
        "[history]\npersistence = \"none\"\n"
    );
    let json = expected("version-out-of-range");
    let tool = &json["expected_report"]["tools"][0];
    assert_eq!(tool["detected_version"], "9.9.9");
    assert_eq!(tool["version_in_range"], false);
    assert_eq!(tool["findings"][0]["status"], "stale-ruleset");
}

#[test]
fn claude_code_unknown_version_pins_non_disclosing_fallback_for_unrecognized_value() {
    let case_root = fixtures_root().join("claude-code/unknown-version");
    let input =
        std::fs::read_to_string(case_root.join("files/home/.claude/settings.json")).unwrap();
    assert!(
        input.contains("\"DISABLE_TELEMETRY\": \"yes\""),
        "the fixture must exercise the unrecognized-value stale branch"
    );

    let expected_text = std::fs::read_to_string(case_root.join("expected.json")).unwrap();
    assert!(
        !expected_text.contains("\"yes\""),
        "the hostile raw value must never enter expected output"
    );
    let json: serde_json::Value = serde_json::from_str(&expected_text).unwrap();
    let tool = &json["expected_report"]["tools"][0];
    assert_eq!(tool["detected_version"], serde_json::Value::Null);
    assert_eq!(tool["version_in_range"], false);
    let finding = tool["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["rule_id"] == "claude-code-telemetry-opt-out-01")
        .expect("claude-code-telemetry-opt-out-01 finding must be present");
    assert_eq!(finding["status"], "stale-ruleset");
    assert_eq!(
        finding["message"],
        "Unverified — last-known rule indicates the configured value cannot be interpreted safely. Observed: unrecognized value (raw value withheld)."
    );
    assert_eq!(finding["observation"], serde_json::Value::Null);
}

#[test]
fn claude_code_out_of_range_fixture_remains_safe_looking_but_stale() {
    let case_root = fixtures_root().join("claude-code/version-out-of-range");
    let input =
        std::fs::read_to_string(case_root.join("files/home/.claude/settings.json")).unwrap();
    assert!(input.contains("\"cleanupPeriodDays\": 20"));
    assert!(input.contains("\"DISABLE_TELEMETRY\": \"1\""));
    let json = expected_for("claude-code", "version-out-of-range");
    let tool = &json["expected_report"]["tools"][0];
    assert_eq!(tool["detected_version"], "9.9.9");
    assert_eq!(tool["version_in_range"], false);
    for finding in tool["findings"].as_array().unwrap() {
        assert_eq!(finding["status"], "stale-ruleset");
    }
}
