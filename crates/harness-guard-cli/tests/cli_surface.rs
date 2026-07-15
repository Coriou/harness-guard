mod common;

use common::*;

#[test]
fn list_shows_detection_only() {
    let output = run_case("hardened", &["list"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("codex"));
    assert!(text.contains("0.144.4"));
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
    let synthetic_home = tempfile::tempdir().unwrap();
    let output = run_with_roots(
        &codex_home,
        &files_root.join("path"),
        synthetic_home.path(),
        &["list"],
    );
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("$CODEX_HOME/config.toml"));
    assert!(!text.contains(&codex_home.to_string_lossy().into_owned()));
    assert!(!text.contains(&synthetic_home.path().to_string_lossy().into_owned()));
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
        "<=0.144.4",
        "verified",
        "limitations",
        "unknown conditions",
        "why it matters",
    ] {
        assert!(
            text.to_lowercase().contains(&needle.to_lowercase()),
            "explain output missing {needle:?}"
        );
    }
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
    assert!(text.contains("ruleset 2026.07.15"));
}

#[test]
fn top_level_version_reports_binary_and_ruleset_separately() {
    let output = run_case("hardened", &["--version"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("harness-guard 0.1.0"));
    assert!(text.contains("ruleset 2026.07.15"));
}

#[test]
fn completions_emit_something() {
    let output = run_case("hardened", &["completions", "bash"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!output.stdout.is_empty());
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
    ] {
        let output = run_case("hardened", &args);
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !text.contains("AI agent security scanner"),
            "forbidden positioning phrase in {args:?}"
        );
    }

    let output = run_case("hardened", &["--help"]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("local, execution-free, per-finding-cited config auditor"),
        "binding positioning phrase must appear in top-level help"
    );
}
