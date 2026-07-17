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
fn agent_guide_carries_positioning_and_no_cadence_claims() {
    let text = std::fs::read_to_string(repo_root().join("docs/agent-guide.md")).unwrap();
    assert!(text.contains("local, execution-free, per-finding-cited config auditor"));
    let forbidden_phrase = ["AI agent", "security scanner"].join(" ");
    assert!(!text.contains(&forbidden_phrase));
    for cadence in [
        "weekly",
        "daily re-verification",
        "continuously verified",
        "always up to date",
    ] {
        assert!(
            !text.to_lowercase().contains(cadence),
            "cadence claim {cadence:?} found"
        );
    }
}
