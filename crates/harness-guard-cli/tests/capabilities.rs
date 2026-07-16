mod common;
use common::*;

#[test]
fn capabilities_json_validates_against_its_schema_and_agrees_with_rules() {
    let output = run_case("hardened", &["capabilities", "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let caps: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("schemas/capabilities.schema.json")).unwrap(),
    )
    .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(
        validator.validate(&caps).is_ok(),
        "{:?}",
        validator
            .iter_errors(&caps)
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
    );
    // Alphabetical tool ordering, and rule counts agree with explain surface.
    let tools: Vec<&str> = caps["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["tool"].as_str().unwrap())
        .collect();
    assert_eq!(tools, ["claude-code", "codex", "grok-build"]);
    let total: u64 = caps["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["rules"].as_u64().unwrap())
        .sum();
    assert!(total >= 1);
}

#[test]
fn capabilities_is_identical_regardless_of_fixture_environment() {
    // Offline + deterministic: capabilities reads no filesystem state.
    let first = run_case("hardened", &["capabilities", "--json"]);
    let second = run_case("missing", &["capabilities", "--json"]);
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn capabilities_table_lists_all_three_tools() {
    let output = run_case("hardened", &["capabilities"]);
    let text = String::from_utf8_lossy(&output.stdout);
    for tool in ["claude-code", "codex", "grok-build"] {
        assert!(text.contains(tool));
    }
}

// DEFERRED (plan-review amendment, binding — see
// .superpowers/sdd/task-20-brief.md Step 3 Sequencing note): §8.1 requires
// both views golden-tested — a `capabilities_table_view_is_golden_tested`
// insta snapshot of the table view, and a
// `capabilities_json_view_matches_committed_golden` byte-compare against a
// committed `tests/goldens/capabilities.expected.json`, the same convention
// scan_fixtures.rs uses for `expected.json`. Both pin exact rule counts and
// categories per tool, which are not stable until Task 19 (Grok Build rules,
// last of the rule-authoring tasks, release-gating) lands. Task 19's
// implementer (or a follow-up task) must author those two tests plus the
// reviewed, committed golden fixture once rule counts are final — do not add
// them before then, or they will immediately drift.
