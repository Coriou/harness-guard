mod common;

use common::*;

/// Runtime-mutated hostile cases are exercised separately in Task 14.
const CASES: &[(&str, i32)] = &[
    ("missing", 0),
    ("minimal", 1),
    ("hardened", 0),
    ("risky-unset", 1),
    ("risky-explicit", 1),
    ("malformed-toml", 2),
    ("unrecognized-value", 0),
    ("deep-nesting", 2),
    ("unknown-version", 0),
    ("version-out-of-range", 0),
];

#[test]
fn fixture_exit_codes_and_json_goldens() {
    for (case, expected_exit) in CASES {
        let output = run_case(case, &["scan", "--json"]);
        assert_eq!(
            output.status.code(),
            Some(*expected_exit),
            "exit code for {case}"
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{case}: --json must emit valid JSON: {error}"));
        let expected: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                repo_root()
                    .join("fixtures/codex")
                    .join(case)
                    .join("expected.json"),
            )
            .expect("fixture golden is readable"),
        )
        .expect("fixture golden is JSON");
        assert_json_subset(&expected["expected_report"], &report, case);
    }
}

#[test]
fn json_report_validates_against_report_schema() {
    let output = run_case("risky-unset", &["scan", "--json"]);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("schemas/report.schema.json")).unwrap(),
    )
    .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(
        validator.validate(&report).is_ok(),
        "{:?}",
        validator
            .iter_errors(&report)
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn fail_on_semantics() {
    assert_eq!(
        run_case("risky-unset", &["scan", "--fail-on", "never"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        run_case("unrecognized-value", &["scan"]).status.code(),
        Some(0)
    );
    assert_eq!(
        run_case("unknown-version", &["scan"]).status.code(),
        Some(0)
    );
    assert_eq!(
        run_case("malformed-toml", &["scan", "--fail-on", "never"])
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn unknown_tool_flag_is_usage_error() {
    let output = run_case("hardened", &["scan", "--tool", "cursor"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn scan_accepts_global_color_flag_after_subcommand() {
    assert_eq!(
        run_case("hardened", &["scan", "--color", "never"])
            .status
            .code(),
        Some(0)
    );
}

#[test]
fn raw_values_never_echo_anywhere() {
    for args in [
        vec!["scan"],
        vec!["scan", "--json"],
        vec!["scan", "--verbose"],
    ] {
        let output = run_case("unrecognized-value", &args);
        let all = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!all.contains("archive"), "raw value leaked via {args:?}");
    }
}

#[test]
fn output_paths_are_redacted() {
    let files_root = fixture("risky-unset");
    let output = run_in(&files_root, &["scan", "--json"]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("~/.codex/config.toml"));
    assert!(
        !text.contains(&files_root.to_string_lossy().into_owned()),
        "absolute fixture home leaked"
    );
    assert!(!text.contains("/Users/"), "absolute home path leaked");
}

#[test]
fn explicit_codex_home_outside_home_is_symbolic() {
    let files_root = fixture("risky-unset");
    let codex_home = files_root.join("codex-home");
    let synthetic_home = tempfile::tempdir().unwrap();
    let output = run_with_roots(
        &codex_home,
        &files_root.join("path"),
        synthetic_home.path(),
        &["scan", "--json"],
    );
    assert_eq!(output.status.code(), Some(1));

    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("$CODEX_HOME/config.toml"));
    assert!(
        !text.contains(&codex_home.to_string_lossy().into_owned()),
        "absolute explicit CODEX_HOME leaked"
    );
    assert!(
        !text.contains(&synthetic_home.path().to_string_lossy().into_owned()),
        "absolute HOME leaked"
    );
}
