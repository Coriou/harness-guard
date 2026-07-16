mod common;

use common::*;

/// Runtime-mutated hostile cases are exercised separately in Task 14.
const CASES: &[(&str, i32)] = &[
    ("missing", 0),
    ("minimal", 0),
    ("hardened", 0),
    ("risky-unset", 0),
    ("risky-explicit", 1),
    ("malformed-toml", 2),
    ("unrecognized-value", 0),
    ("deep-nesting", 2),
    ("unknown-version", 0),
    ("version-out-of-range", 0),
];

/// claude-code fixture matrix (Task 18). Runtime-mutated hostile cases
/// (oversized, permission-denied, symlink-config, non-utf8) are exercised
/// separately in hostile.rs, mirroring the codex CASES/hostile.rs split above.
const CLAUDE_CASES: &[(&str, i32)] = &[
    ("missing", 0),
    ("minimal", 0),
    ("hardened", 0),
    ("risky-unset", 0),
    ("risky-explicit", 1),
    ("malformed-json", 2),
    ("unrecognized-value", 0),
    ("deep-nesting", 2),
    ("unknown-version", 0),
    ("version-out-of-range", 0),
    ("duplicate-keys", 1),
    ("float-where-integer", 0),
    ("huge-number", 0),
    ("secret-shaped", 2),
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
fn claude_code_fixture_exit_codes_and_json_goldens() {
    for (case, expected_exit) in CLAUDE_CASES {
        let output = run_harness_case("claude-code", case, &["scan", "--json"]);
        assert_eq!(
            output.status.code(),
            Some(*expected_exit),
            "exit code for claude-code/{case}"
        );
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                panic!("claude-code/{case}: --json must emit valid JSON: {error}")
            });
        let expected: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                repo_root()
                    .join("fixtures/claude-code")
                    .join(case)
                    .join("expected.json"),
            )
            .expect("fixture golden is readable"),
        )
        .expect("fixture golden is JSON");
        assert_json_subset(
            &expected["expected_report"],
            &report,
            &format!("claude-code/{case}"),
        );
    }
}

#[test]
fn mixed_codex_pass_claude_degraded_exit_code_and_json_golden() {
    let output = run_mixed_case("codex-pass-claude-degraded", &["scan", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "claude-code degraded ⇒ exit 2 even though codex alone would pass"
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!("mixed/codex-pass-claude-degraded: --json must emit valid JSON: {error}")
        });
    let expected: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            repo_root()
                .join("fixtures/mixed/codex-pass-claude-degraded")
                .join("expected.json"),
        )
        .expect("mixed fixture golden is readable"),
    )
    .expect("mixed fixture golden is JSON");
    assert_json_subset(
        &expected["expected_report"],
        &report,
        "mixed/codex-pass-claude-degraded",
    );
    assert_eq!(report["tools"].as_array().unwrap().len(), 2);
    assert_eq!(report["tools"][0]["tool"], "claude-code");
    assert_eq!(report["tools"][1]["tool"], "codex");
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
fn json_platform_matches_the_supported_build_target() {
    let output = run_case("hardened", &["scan", "--json"]);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    #[cfg(target_os = "macos")]
    let expected = "macos";
    #[cfg(target_os = "linux")]
    let expected = "linux";

    assert_eq!(report["platform"]["os"], expected);
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
fn tool_flag_accepts_all_three_ids() {
    for tool in ["codex", "claude-code", "grok-build"] {
        let output = run_case("hardened", &["scan", "--tool", tool, "--json"]);
        assert!(
            matches!(output.status.code(), Some(0) | Some(1)),
            "{tool} must be a valid --tool"
        );
    }
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
fn secret_shaped_hostile_token_never_echoes_in_any_output_mode() {
    // The claude-code secret-shaped fixture's committed settings.json embeds
    // a secret-looking bareword token that makes the file invalid JSON
    // syntax; the categorical parse diagnostic and every finding must never
    // surface the token itself, only the structural error category.
    for args in [
        vec!["scan"],
        vec!["scan", "--json"],
        vec!["scan", "--verbose"],
    ] {
        let output = run_harness_case("claude-code", "secret-shaped", &args);
        let all = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !all.contains("xk-hostile"),
            "secret-shaped token leaked via {args:?}"
        );
        assert!(
            !all.contains("1234567890abcdef"),
            "secret-shaped token fragment leaked via {args:?}"
        );
    }
}

#[test]
fn output_paths_are_redacted() {
    let files_root = fixture("risky-unset");
    let output = run_in(&files_root, &["scan", "--json"]);
    let text = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["tools"][0]["config_paths"][0],
        "~/codex-home/config.toml"
    );
    assert!(
        !text.contains(&files_root.to_string_lossy().into_owned()),
        "absolute fixture home leaked"
    );
    assert!(!text.contains("/Users/"), "absolute home path leaked");
}

#[test]
fn explicit_codex_home_outside_home_is_symbolic() {
    let files_root = fixture("risky-explicit");
    let codex_home = files_root.join("codex-home");
    let synthetic_home_dir = tempfile::tempdir().unwrap();
    // Canonicalize: on macOS, `$TMPDIR` (and therefore a bare `tempdir()`
    // path) is nested under `/var`, itself a symlink to `/private/var`. The
    // hardened no-follow readers correctly refuse any ancestor-path symlink,
    // so an uncanonicalized HOME would make claude-code/grok-build's
    // `HOME/.claude` and `HOME/.grok` probes come back `Refused` (degraded)
    // instead of `Missing` (undetected) now that scan covers every harness
    // by default — matching the established pattern used throughout the
    // unit tests (`dir.path().canonicalize().unwrap()`).
    let synthetic_home = synthetic_home_dir.path().canonicalize().unwrap();
    let output = run_with_roots(
        &codex_home,
        &files_root.join("path"),
        &synthetic_home,
        &["scan", "--json"],
    );
    assert_eq!(output.status.code(), Some(1));

    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let rendered_path = report["tools"][0]["config_paths"][0]
        .as_str()
        .expect("config path is rendered as a string");
    let remediation = report["tools"][0]["findings"][0]["remediation"]["command"]
        .as_str()
        .expect("finding has remediation text");
    assert!(
        rendered_path == "$CODEX_HOME/config.toml" || rendered_path.starts_with("~/"),
        "config path must use a safe symbolic root, got {rendered_path:?}"
    );

    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        !text.contains(&codex_home.to_string_lossy().into_owned()),
        "absolute explicit CODEX_HOME leaked"
    );
    assert!(
        !text.contains(&synthetic_home.to_string_lossy().into_owned()),
        "absolute HOME leaked"
    );
    assert!(
        remediation.contains("CODEX_HOME/config.toml"),
        "remediation must remain correct for a custom CODEX_HOME: {remediation:?}"
    );
}

#[test]
fn no_absolute_path_escapes_the_fixture_tree_for_any_harness() {
    // §5.1: extends the existing real-config protection to ~/.claude and
    // ~/.grok, which also exist on dev machines. The scan runs with every
    // ambient variable cleared and all three homes inside the fixture; no
    // absolute path outside the fixture may appear in any output.
    //
    // This run only covers the codex `hardened` fixture — claude-code and
    // grok-build are not detected in it, so their config-path redaction is
    // not yet exercised here.
    let files_root = fixture("hardened");
    let output = run_in(&files_root, &["scan", "--json", "--verbose"]);
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !all.contains("/Users/"),
        "home-anchored absolute path leaked"
    );
    assert!(
        !all.contains(&files_root.to_string_lossy().into_owned()),
        "fixture path leaked"
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    for tool in report["tools"].as_array().unwrap() {
        for path in tool["config_paths"].as_array().unwrap() {
            let rendered = path.as_str().unwrap();
            assert!(
                rendered.starts_with('~') || rendered.starts_with('$'),
                "config path {rendered:?} must have a symbolic root"
            );
        }
    }

    // Task 18: extend the same protection over `fixtures/mixed/codex-pass-
    // claude-degraded`, the one fixture whose synthetic home contains BOTH a
    // .codex store and a .claude store, so claude-code's config-path
    // redaction is exercised here for the first time.
    let mixed_output = run_mixed_case(
        "codex-pass-claude-degraded",
        &["scan", "--json", "--verbose"],
    );
    let mixed_all = format!(
        "{}{}",
        String::from_utf8_lossy(&mixed_output.stdout),
        String::from_utf8_lossy(&mixed_output.stderr)
    );
    assert!(
        !mixed_all.contains("/Users/"),
        "home-anchored absolute path leaked (mixed fixture)"
    );
    let mixed_files_root = repo_root()
        .join("fixtures/mixed/codex-pass-claude-degraded/files")
        .to_string_lossy()
        .into_owned();
    assert!(
        !mixed_all.contains(&mixed_files_root),
        "fixture path leaked (mixed fixture)"
    );
    let mixed_report: serde_json::Value = serde_json::from_slice(&mixed_output.stdout).unwrap();
    let mixed_tools = mixed_report["tools"].as_array().unwrap();
    assert_eq!(mixed_tools.len(), 2, "mixed fixture must detect both tools");
    for tool in mixed_tools {
        for path in tool["config_paths"].as_array().unwrap() {
            let rendered = path.as_str().unwrap();
            assert!(
                rendered.starts_with('~') || rendered.starts_with('$'),
                "config path {rendered:?} must have a symbolic root (mixed fixture)"
            );
        }
    }
}
