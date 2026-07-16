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
    let path = fixtures_root()
        .join("codex")
        .join(case)
        .join("expected.json");
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
    let mut stack = vec![fixtures_root().join("codex")];
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
    let finding = &tool["findings"][0];
    assert_eq!(tool["detected_version"], serde_json::Value::Null);
    assert_eq!(tool["version_in_range"], false);
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
