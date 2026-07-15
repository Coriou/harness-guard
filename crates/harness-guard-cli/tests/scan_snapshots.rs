#[allow(dead_code)]
mod common;

use common::*;

fn snap(case: &str, args: &[&str], name: &str) {
    let out = run_case(case, args);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    insta::with_settings!({filters => vec![
        // Timestamps vary per run.
        (r"\d{4}-\d{2}-\d{2}T[0-9:.+\-Z]+", "[TIMESTAMP]"),
        // Fixture paths under the synthetic test home vary per checkout.
        (r"~[^\s]*codex-home[^\s]*", "[CONFIG_PATH]"),
    ]}, {
        insta::assert_snapshot!(name, text);
    });
}

#[test]
fn term_risky_unset() {
    snap("risky-unset", &["scan"], "risky_unset");
}

#[test]
fn term_hardened_verbose() {
    snap("hardened", &["scan", "--verbose"], "hardened_verbose");
}

#[test]
fn term_hardened_default_hides_pass_blocks() {
    let out = run_case("hardened", &["scan"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        !text.contains("PASS:"),
        "default output shows passes only as a count"
    );
    assert!(text.contains("1 passed"));
}

#[test]
fn term_unrecognized_value() {
    snap("unrecognized-value", &["scan"], "unknown_value");
}

#[test]
fn term_unknown_version_banner() {
    snap("unknown-version", &["scan"], "stale_banner");
}

#[test]
fn term_version_out_of_range() {
    snap("version-out-of-range", &["scan"], "stale_out_of_range");
}

#[test]
fn term_missing() {
    snap("missing", &["scan"], "missing");
}

#[test]
fn term_quiet() {
    snap("risky-unset", &["scan", "--quiet"], "risky_unset_quiet");
}

#[test]
fn min_severity_never_hides_unknown_or_stale() {
    let out = run_case("unrecognized-value", &["scan", "--min-severity", "warning"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("?? UNKNOWN"),
        "--min-severity must not hide unknown blocks"
    );
    let out = run_case("unknown-version", &["scan", "--min-severity", "warning"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("UNVERIFIED (stale ruleset)"));
}

#[test]
fn citations_appear_in_default_output() {
    let out = run_case("risky-unset", &["scan"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("= source: https://"),
        "citation must be in DEFAULT output"
    );
    assert!(text.contains('('), "retrieved date shown with the citation");
    assert!(text.contains("= harness-guard explain codex-history-persist-01"));
    assert!(text.contains("No numeric score is produced"));
    assert!(text.contains("no network requests made"));
}

#[test]
fn detection_confidence_is_lowercase() {
    let out = run_case("risky-unset", &["scan"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("confidence high"));
    assert!(!text.contains("confidence High"));
}
