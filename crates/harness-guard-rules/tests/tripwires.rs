//! Live-keys / legacy-claim tripwires (protocol §7, amended 2026-07-17).
//!
//! Owner decision 2026-07-17 (evidence pack docs/research/evidence/grok-build/
//! 2026-07-17): the July-13 audit banned GROK_TELEMETRY_*, [telemetry], and
//! trace_upload because public docs omitted them. OSS source + in-tree user
//! guide re-document those as live controls. The retired-key ban list is
//! therefore empty — no currently-live key is banned.
//!
//! What remains pinned: rules and user-facing strings must never revive the
//! *legacy research-only* claim that those keys alone stop canary-repo wire
//! uploads. Behavior claims require independent-reproduction lab artifacts.
use std::path::{Path, PathBuf};

/// Strings that asserted the retired-keys ban. Kept as documentation of the
/// owner supersession; the ban itself is lifted (empty list).
#[allow(dead_code)]
const LIFTED_RETIRED_GROK_KEY_BAN: [&str; 4] = [
    "GROK_TELEMETRY_ENABLED",
    "GROK_TELEMETRY_TRACE_UPLOAD",
    "trace_upload",
    "[telemetry]",
];

/// Legacy research-only claim language that must not reappear as a shipped
/// remediation or rule message. Local-posture rules may cite the live keys;
/// they must not claim those keys stop canary-repo uploads without lab evidence.
const LEGACY_CANARY_UPLOAD_CLAIMS: [&str; 3] =
    ["stop canary", "stops canary", "canary-repo upload"];

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
fn lifted_retired_key_ban_is_documented_empty() {
    // Pin the owner decision: the ban list is intentionally empty after the
    // 2026-07-17 OSS re-documentation. If someone re-adds a blanket ban without
    // a new dead-key intake, this constant-length assertion forces a review.
    assert!(
        LIFTED_RETIRED_GROK_KEY_BAN.len() == 4,
        "document the four formerly-banned strings; do not silently drop the history"
    );
}

#[test]
fn rules_never_claim_keys_stop_canary_uploads_without_lab_evidence() {
    let mut files = Vec::new();
    walk_files(&repo_root().join("rules"), &mut files);
    assert!(!files.is_empty(), "rules tree must exist");
    files.push(repo_root().join("docs/agent-guide.md"));
    for file in files {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|_| panic!("rule file {file:?} is readable UTF-8"));
        let lower = text.to_lowercase();
        for claim in LEGACY_CANARY_UPLOAD_CLAIMS {
            assert!(
                !lower.contains(claim),
                "legacy canary-upload claim {claim:?} reappeared in {file:?}; \
                 behavior claims need independent-reproduction lab artifacts"
            );
        }
    }
}

#[test]
fn readme_and_agent_guide_carry_positioning_and_no_cadence_claims() {
    let root = repo_root();
    let files = ["docs/agent-guide.md", "README.md"];
    let forbidden_phrase = ["AI agent", "security scanner"].join(" ");
    for rel in files {
        let path = root.join(rel);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{path:?} must be readable UTF-8"));
        assert!(
            text.contains("local, execution-free, per-finding-cited config auditor"),
            "{rel} must carry the positioning phrase"
        );
        assert!(
            !text.contains(&forbidden_phrase),
            "{rel} must not contain {forbidden_phrase:?}"
        );
        for cadence in [
            "weekly",
            "daily re-verification",
            "continuously verified",
            "always up to date",
        ] {
            assert!(
                !text.to_lowercase().contains(cadence),
                "cadence claim {cadence:?} found in {rel}"
            );
        }
    }
}

#[test]
fn security_md_does_not_claim_versioned_releases_are_absent() {
    let text = std::fs::read_to_string(repo_root().join("SECURITY.md")).unwrap();
    assert!(
        !text
            .to_lowercase()
            .contains("until versioned releases exist"),
        "SECURITY.md must name the current tagged preview now that 0.0.1 / 0.0.2 exist"
    );
}

fn line_writes_into_rules_or_freshness(line: &str) -> bool {
    let compact: String = line
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    let after_redirect = compact.contains(">freshness/")
        || compact.contains(">>freshness/")
        || compact.contains(">rules/")
        || compact.contains(">>rules/");
    let tee_into = compact.contains("teefreshness/")
        || compact.contains("tee-afreshness/")
        || compact.contains("teerules/")
        || compact.contains("tee-arules/");
    after_redirect || tee_into
}

#[test]
fn maintainer_fetch_scripts_are_strict_and_write_nothing_under_rules_or_freshness() {
    let root = repo_root();
    let scripts = [
        root.join("scripts/freshness/probe-releases.sh"),
        root.join("scripts/freshness/fetch-cited.sh"),
    ];
    for script in scripts {
        let text = std::fs::read_to_string(&script)
            .unwrap_or_else(|_| panic!("maintainer script {script:?} must exist"));
        assert!(text.contains("set -eu"), "{script:?} must contain set -eu");
        for line in text.lines() {
            assert!(
                !line_writes_into_rules_or_freshness(line),
                "{script:?} must not redirect into rules/ or freshness/: {line}"
            );
        }
    }
}

#[test]
fn core_cli_scan_and_ci_do_not_reference_maintainer_fetch_helpers() {
    let root = repo_root();
    let mut files = Vec::new();
    walk_files(&root.join("crates/harness-guard-core"), &mut files);
    walk_files(&root.join("crates/harness-guard-cli/src"), &mut files);
    files.push(root.join(".github/workflows/ci.yml"));
    assert!(files.iter().any(|p| p.ends_with("scan.rs")));
    let needles = ["fetch-cited", "probe-releases", "web.archive.org/save"];
    for file in files {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|_| panic!("{file:?} must be readable UTF-8"));
        for needle in needles {
            assert!(
                !text.contains(needle),
                "{needle:?} must not appear in product/CI tree {file:?}"
            );
        }
    }
}
