//! Retired-mitigation tripwires (spec §7.3.7). These strings were mitigation
//! keys for an old Grok Build version and must never reappear in any rule,
//! remediation, or user-facing string. Same mechanism as the forbidden
//! positioning-phrase test.
use std::path::{Path, PathBuf};

const RETIRED_GROK_KEYS: [&str; 4] = [
    "GROK_TELEMETRY_ENABLED",
    "GROK_TELEMETRY_TRACE_UPLOAD",
    "trace_upload",
    "[telemetry]",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

#[test]
fn retired_grok_keys_never_reappear_in_rules() {
    let mut files = Vec::new();
    walk_files(&repo_root().join("rules"), &mut files);
    assert!(!files.is_empty(), "rules tree must exist");
    for file in files {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|_| panic!("rule file {file:?} is readable UTF-8"));
        for key in RETIRED_GROK_KEYS {
            assert!(
                !text.contains(key),
                "retired Grok mitigation key {key:?} reappeared in {file:?}"
            );
        }
    }
}
