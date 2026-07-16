mod common;

use common::*;

#[test]
fn list_shows_detection_only() {
    let output = run_case("hardened", &["list"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("codex"));
    assert!(text.contains("0.144.5"));
    assert!(
        text.contains("claude-code"),
        "list must enumerate claude-code even when undetected"
    );
    assert!(
        text.contains("grok-build"),
        "list must enumerate grok-build even when undetected"
    );
    assert!(
        !text.contains("codex-history-persist-01"),
        "list must never evaluate rules"
    );
    assert!(!text.contains("/Users/"), "paths must be redacted");
}

#[test]
fn list_reports_version_not_detected() {
    let output = run_case("unknown-version", &["list"]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("version not detected"));
}

#[test]
fn list_symbolically_redacts_explicit_codex_home() {
    let files_root = fixture("hardened");
    let codex_home = files_root.join("codex-home");
    let synthetic_home_dir = tempfile::tempdir().unwrap();
    // Canonicalize for the same reason as scan_fixtures.rs's
    // explicit_codex_home_outside_home_is_symbolic: an uncanonicalized
    // tempdir HOME on macOS is nested under the `/var` -> `/private/var`
    // symlink, which the hardened no-follow probes correctly refuse.
    let synthetic_home = synthetic_home_dir.path().canonicalize().unwrap();
    let output = run_with_roots(
        &codex_home,
        &files_root.join("path"),
        &synthetic_home,
        &["list"],
    );
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("$CODEX_HOME/config.toml") || text.contains("~/"),
        "config path must have a symbolic root: {text}"
    );
    assert!(!text.contains(&codex_home.to_string_lossy().into_owned()));
    assert!(!text.contains(&synthetic_home.to_string_lossy().into_owned()));
}

#[test]
fn list_path_only_version_detection_has_medium_confidence() {
    let synthetic = tempfile::tempdir().unwrap();
    let base = synthetic.path().canonicalize().unwrap();
    let path = base.join("path");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("codex"), "synthetic marker; never executed").unwrap();
    std::fs::write(
        path.join("package.json"),
        r#"{"name":"@openai/codex","version":"0.144.5"}"#,
    )
    .unwrap();

    let output = run_with_roots(
        &base.join("absent-codex-home"),
        &path,
        &base.join("home"),
        &["list"],
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0));
    assert!(text.contains("0.144.5"));
    assert!(text.contains("medium"));
}

#[test]
fn list_path_marker_without_version_or_home_has_low_confidence() {
    let synthetic = tempfile::tempdir().unwrap();
    let base = synthetic.path().canonicalize().unwrap();
    let path = base.join("path");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("codex"), "synthetic marker; never executed").unwrap();

    let output = run_with_roots(
        &base.join("absent-codex-home"),
        &path,
        &base.join("home"),
        &["list"],
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0));
    assert!(text.contains("version not detected"));
    assert!(text.contains("low"));
}

#[test]
fn explain_shows_full_evidence_record() {
    let output = run_case("hardened", &["explain", "codex-history-persist-01"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "codex-history-persist-01",
        "official-documentation",
        "content_hash",
        "sha256:",
        "retrieved",
        "archived",
        "web.archive.org",
        "tested versions",
        "<=0.144.5",
        "verified",
        "limitations",
        "unknown conditions",
        "why it matters",
        "confidence: high",
    ] {
        assert!(
            text.to_lowercase().contains(&needle.to_lowercase()),
            "explain output missing {needle:?}"
        );
    }
    assert_eq!(
        text.matches("confidence:").count(),
        4,
        "each outcome must state its confidence explicitly"
    );
}

#[test]
fn explain_unknown_rule_suggests_nearest_and_exits_2() {
    let output = run_case("hardened", &["explain", "codex-history-persist-02"]);
    assert_eq!(output.status.code(), Some(2));
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains("codex-history-persist-01"),
        "nearest-match suggestion expected"
    );
}

#[test]
fn version_reports_binary_and_ruleset_separately() {
    let output = run_case("hardened", &["version"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("harness-guard 0.1.0"));
    assert!(text.contains("ruleset 2026.07.16"));
}

#[test]
fn top_level_version_reports_binary_and_ruleset_separately() {
    let output = run_case("hardened", &["--version"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("harness-guard 0.1.0"));
    assert!(text.contains("ruleset 2026.07.16"));
}

#[test]
fn bash_completions_have_reachable_subcommand_states() {
    let output = run_case("hardened", &["completions", "bash"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("cmd=\"harness__guard__subcmd__scan\""));
    assert!(text.contains("harness__guard__subcmd__scan)"));
    assert!(!text.contains("harness__subcmd__guard"));
}

#[test]
fn scan_accepts_color_never_after_subcommand() {
    assert_eq!(
        run_case("hardened", &["scan", "--color", "never"])
            .status
            .code(),
        Some(0)
    );
}

#[test]
fn help_uses_positioning_and_never_the_forbidden_phrase() {
    for args in [
        vec!["--help"],
        vec!["scan", "--help"],
        vec!["list", "--help"],
        vec!["explain", "--help"],
        vec!["version", "--help"],
        vec!["completions", "--help"],
    ] {
        let output = run_case("hardened", &args);
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let forbidden = ["AI agent", "security scanner"].join(" ");
        assert!(
            !text.contains(&forbidden),
            "forbidden positioning phrase in {args:?}"
        );
        let examples = text
            .find("Examples:")
            .unwrap_or_else(|| panic!("Examples block missing from {args:?}"));
        let usage = text
            .find("Usage:")
            .unwrap_or_else(|| panic!("Usage block missing from {args:?}"));
        assert!(examples < usage, "Examples must precede Usage in {args:?}");
    }

    let output = run_case("hardened", &["--help"]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("local, execution-free, per-finding-cited config auditor"),
        "binding positioning phrase must appear in top-level help"
    );
}

#[test]
fn retired_grok_keys_never_appear_in_cli_output() {
    // Spec §7.3.7: the tripwire covers user-facing output corpora, not just
    // rule files. Help text and a fixture scan are the output corpus.
    let retired = [
        "GROK_TELEMETRY_ENABLED",
        "GROK_TELEMETRY_TRACE_UPLOAD",
        "trace_upload",
        "[telemetry]",
    ];
    for args in [
        vec!["--help"],
        vec!["scan", "--help"],
        vec!["scan", "--json"],
        vec!["scan", "--verbose"],
        vec!["explain", "codex-history-persist-01"],
    ] {
        let output = run_case("hardened", &args);
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        for key in retired {
            assert!(
                !text.contains(key),
                "retired key {key:?} in output of {args:?}"
            );
        }
    }
}
