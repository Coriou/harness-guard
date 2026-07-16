//! §6.5: embedded rules correspond 1:1 with the on-disk rules/ tree, every
//! embedded rule validates, and a rule in rules/<tool>/ declares that tool.
use harness_guard_rules::loader::load_rules;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn on_disk_rule_ids() -> BTreeSet<(String, String)> {
    // (tool-dir, rule id) pairs discovered by walking rules/.
    let mut ids = BTreeSet::new();
    let rules_dir = repo_root().join("rules");
    for tool_entry in std::fs::read_dir(&rules_dir).unwrap() {
        let tool_path = tool_entry.unwrap().path();
        if !tool_path.is_dir() {
            continue;
        }
        let tool_dir = tool_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        for rule_entry in std::fs::read_dir(&tool_path).unwrap() {
            let rule_path = rule_entry.unwrap().path();
            if rule_path.extension().is_some_and(|ext| ext == "json") {
                let json: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&rule_path).unwrap()).unwrap();
                ids.insert((tool_dir.clone(), json["id"].as_str().unwrap().to_string()));
            }
        }
    }
    ids
}

#[test]
fn embedded_rules_match_the_on_disk_tree_one_to_one() {
    let disk = on_disk_rule_ids();
    let embedded: BTreeSet<(String, String)> = load_rules()
        .iter()
        .map(|rule| (rule.raw().tool.clone(), rule.raw().id.clone()))
        .collect();
    // Path consistency: the tool directory IS the declared tool, so comparing
    // (dir, id) with (tool, id) proves both 1:1 embedding and §5.6 path
    // consistency in one assertion.
    assert_eq!(disk, embedded);
}

#[test]
fn every_embedded_rule_id_is_prefixed_with_its_tool_id() {
    for rule in load_rules() {
        let raw = rule.raw();
        assert!(
            raw.id.starts_with(&format!("{}-", raw.tool)),
            "rule id {} must be prefixed with its tool id {}",
            raw.id,
            raw.tool
        );
    }
}
