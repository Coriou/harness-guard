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
    assert!(
        text.contains("$CODEX_HOME/config.toml") || text.contains("~/"),
        "config path must have a symbolic root: {text}"
    );
    assert!(!text.contains(&codex_home.to_string_lossy().into_owned()));
    assert!(!text.contains(&synthetic_home.path().to_string_lossy().into_owned()));
}

#[test]
fn list_path_only_version_detection_has_medium_confidence() {
    let synthetic = tempfile::tempdir().unwrap();
    let path = synthetic.path().join("path");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("codex"), "synthetic marker; never executed").unwrap();
    std::fs::write(
        path.join("package.json"),
        r#"{"name":"@openai/codex","version":"0.144.4"}"#,
    )
    .unwrap();

    let output = run_with_roots(
        &synthetic.path().join("absent-codex-home"),
        &path,
        &synthetic.path().join("home"),
        &["list"],
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0));
    assert!(text.contains("0.144.4"));
    assert!(text.contains("medium"));
}

#[test]
fn list_path_marker_without_version_or_home_has_low_confidence() {
    let synthetic = tempfile::tempdir().unwrap();
    let path = synthetic.path().join("path");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("codex"), "synthetic marker; never executed").unwrap();

    let output = run_with_roots(
        &synthetic.path().join("absent-codex-home"),
        &path,
        &synthetic.path().join("home"),
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
        "<=0.144.4",
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
        3,
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
