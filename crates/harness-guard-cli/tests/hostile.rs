mod common;

use common::*;
use std::path::{Path, PathBuf};

/// Copy a fixture's files into a tempdir so runtime mutation never touches
/// the committed tree (and absolutely never the real ~/.codex).
fn temp_copy(case: &str) -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("files");
    copy_dir(&fixture(case), &destination);
    let destination = destination.canonicalize().unwrap();
    (temp, destination)
}

/// Same as `temp_copy` but rooted at `fixtures/<tool>/<case>/files` (Task 18),
/// reusing the same copy_dir safety guarantees (no symlinks copied) for the
/// new-harness fixture layout.
fn temp_copy_harness(tool: &str, case: &str) -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("files");
    copy_dir(&harness_fixture(tool, case), &destination);
    let destination = destination.canonicalize().unwrap();
    (temp, destination)
}

/// Run against a temp-copied new-harness `files` tree (home/, path/), mirroring
/// `run_harness_case`'s root construction: CODEX_HOME stays absent so codex
/// never interferes.
fn run_harness_files(files: &Path, args: &[&str]) -> std::process::Output {
    let home = files.join("home");
    run_with_roots(
        &home.join("absent-codex-home"),
        &files.join("path"),
        &home,
        args,
    )
}

fn copy_dir(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let metadata = std::fs::symlink_metadata(entry.path()).unwrap();
        let destination_entry = destination.join(entry.file_name());
        assert!(
            !metadata.file_type().is_symlink(),
            "synthetic fixture trees must not contain symlinks"
        );
        if metadata.is_dir() {
            copy_dir(&entry.path(), &destination_entry);
        } else if metadata.is_file() {
            std::fs::copy(entry.path(), destination_entry).unwrap();
        } else {
            panic!("synthetic fixture trees must contain only regular files and directories");
        }
    }
}

#[cfg(unix)]
#[test]
fn fixture_copy_rejects_symlinks_without_copying_the_target() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let destination = temp.path().join("destination");
    let external_sentinel = temp.path().join("external-sentinel");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(&external_sentinel, "must never be copied").unwrap();
    std::os::unix::fs::symlink(&external_sentinel, source.join("escape")).unwrap();

    let result = std::panic::catch_unwind(|| copy_dir(&source, &destination));

    assert!(result.is_err(), "fixture symlink must be rejected");
    assert!(
        !destination.join("escape").exists(),
        "the symlink target must not be copied into the destination"
    );
    assert_eq!(
        std::fs::read_to_string(&external_sentinel).unwrap(),
        "must never be copied",
        "the external sentinel must remain unchanged"
    );
}

fn expected_report(case: &str) -> serde_json::Value {
    let expected: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            repo_root()
                .join("fixtures/codex")
                .join(case)
                .join("expected.json"),
        )
        .expect("hostile fixture golden is readable"),
    )
    .expect("hostile fixture golden is JSON");
    expected["expected_report"].clone()
}

fn expected_report_for(tool: &str, case: &str) -> serde_json::Value {
    let expected: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            repo_root()
                .join("fixtures")
                .join(tool)
                .join(case)
                .join("expected.json"),
        )
        .expect("hostile fixture golden is readable"),
    )
    .expect("hostile fixture golden is JSON");
    expected["expected_report"].clone()
}

fn json_report(output: &std::process::Output, case: &str) -> serde_json::Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{case}: --json must emit valid JSON: {error}"))
}

#[cfg(unix)]
#[test]
fn symlink_config_is_not_followed() {
    let (_temp, files) = temp_copy("symlink-config");
    let home = files.join("codex-home");
    std::os::unix::fs::symlink(home.join("real-config.toml"), home.join("config.toml")).unwrap();

    let output = run_in(&files, &["scan", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "refused read degrades the scan"
    );
    let report = json_report(&output, "symlink-config");
    assert_json_subset(
        &expected_report("symlink-config"),
        &report,
        "symlink-config",
    );
    assert_eq!(report["tools"][0]["findings"][0]["status"], "unknown");
    let reason = report["tools"][0]["findings"][0]["unknown_reason"]
        .as_str()
        .unwrap();
    assert!(reason.contains("symlink"));
}

#[cfg(unix)]
#[test]
fn symlinked_codex_home_is_not_followed() {
    let (_temp, files) = temp_copy("symlink-config");
    let linked_home = files.join("codex-home");
    std::fs::rename(
        linked_home.join("real-config.toml"),
        linked_home.join("config.toml"),
    )
    .unwrap();
    let real_home = files.join("real-codex-home");
    std::fs::rename(&linked_home, &real_home).unwrap();
    std::os::unix::fs::symlink(&real_home, &linked_home).unwrap();

    let output = run_with_roots(
        &linked_home,
        &files.join("path"),
        &files,
        &["scan", "--json"],
    );
    assert_eq!(output.status.code(), Some(2));
    let report = json_report(&output, "symlinked-codex-home");
    assert_eq!(report["tools"][0]["findings"][0]["status"], "unknown");
    assert!(
        report["tools"][0]["findings"][0]["unknown_reason"]
            .as_str()
            .unwrap()
            .contains("symlink")
    );
}

#[cfg(unix)]
#[test]
fn symlinked_codex_home_ancestor_is_not_followed() {
    let (_temp, files) = temp_copy("symlink-config");
    let original_home = files.join("codex-home");
    std::fs::rename(
        original_home.join("real-config.toml"),
        original_home.join("config.toml"),
    )
    .unwrap();
    let real_parent = files.join("real-parent");
    std::fs::create_dir(&real_parent).unwrap();
    let real_home = real_parent.join("codex-home");
    std::fs::rename(&original_home, &real_home).unwrap();
    let linked_parent = files.join("linked-parent");
    std::os::unix::fs::symlink(&real_parent, &linked_parent).unwrap();
    let linked_home = linked_parent.join("codex-home");

    let output = run_with_roots(
        &linked_home,
        &files.join("path"),
        &files,
        &["scan", "--json"],
    );
    assert_eq!(output.status.code(), Some(2));
    let report = json_report(&output, "symlinked-codex-home-ancestor");
    assert_eq!(report["tools"][0]["findings"][0]["status"], "unknown");
    assert!(
        report["tools"][0]["findings"][0]["unknown_reason"]
            .as_str()
            .unwrap()
            .contains("symlink")
    );
}

#[test]
fn oversized_config_is_refused() {
    let (_temp, files) = temp_copy("oversized");
    let config = files.join("codex-home/config.toml");
    let mut oversized = String::with_capacity(1_100_000);
    while oversized.len() <= 1024 * 1024 {
        oversized.push_str("# synthetic padding line\n");
    }
    std::fs::write(config, oversized).unwrap();

    let output = run_in(&files, &["scan", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let report = json_report(&output, "oversized");
    assert_json_subset(&expected_report("oversized"), &report, "oversized");
    assert_eq!(report["tools"][0]["findings"][0]["status"], "unknown");
}

#[cfg(unix)]
struct RestorePermissions {
    path: PathBuf,
    permissions: std::fs::Permissions,
}

#[cfg(unix)]
impl Drop for RestorePermissions {
    fn drop(&mut self) {
        let _ = std::fs::set_permissions(&self.path, self.permissions.clone());
    }
}

#[cfg(unix)]
#[test]
fn permission_denied_degrades_to_unknown() {
    use std::os::unix::fs::PermissionsExt;

    let (_temp, files) = temp_copy("permission-denied");
    let config = files.join("codex-home/config.toml");
    let restore = RestorePermissions {
        path: config.clone(),
        permissions: std::fs::metadata(&config).unwrap().permissions(),
    };
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o000)).unwrap();

    let output = run_in(&files, &["scan", "--json"]);
    drop(restore);

    assert_eq!(output.status.code(), Some(2));
    let report = json_report(&output, "permission-denied");
    assert_json_subset(
        &expected_report("permission-denied"),
        &report,
        "permission-denied",
    );
    let reason = report["tools"][0]["findings"][0]["unknown_reason"]
        .as_str()
        .unwrap();
    assert!(reason.contains("permission"));
}

#[test]
fn malformed_toml_diagnostic_has_line_col_but_never_content_or_absolute_path() {
    let files = fixture("malformed-toml");
    let output = run_case("malformed-toml", &["scan"]);
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("line 1") && stderr.contains("column"),
        "diagnostic must carry structural line/column: {stderr}"
    );
    assert!(
        !stderr.contains("persistence") && !stderr.contains("none"),
        "config content leaked into diagnostic: {stderr}"
    );
    assert!(
        !stderr.contains(&files.to_string_lossy().into_owned()),
        "absolute fixture path leaked into diagnostic: {stderr}"
    );
    assert!(
        stderr.contains("~/codex-home/config.toml") || stderr.contains("$CODEX_HOME/config.toml"),
        "diagnostic should name only a symbolic config path: {stderr}"
    );
    assert!(
        !output.stdout.is_empty(),
        "degraded scan must still render a report"
    );
}

// --- Task 18: claude-code hostile runtime mutations, mirroring the codex
// cases above via the same temp_copy/RestorePermissions machinery, now
// pointed at the claude-code fixture tree. ---

#[test]
fn claude_code_oversized_config_is_refused() {
    let (_temp, files) = temp_copy_harness("claude-code", "oversized");
    let config = files.join("home/.claude/settings.json");
    let mut oversized = String::with_capacity(1_100_000);
    while oversized.len() <= 1024 * 1024 {
        oversized.push_str("// synthetic padding line\n");
    }
    std::fs::write(config, oversized).unwrap();

    let output = run_harness_files(&files, &["scan", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let report = json_report(&output, "claude-code/oversized");
    assert_json_subset(
        &expected_report_for("claude-code", "oversized"),
        &report,
        "claude-code/oversized",
    );
    assert_eq!(report["tools"][0]["findings"][0]["status"], "unknown");
}

#[cfg(unix)]
#[test]
fn claude_code_permission_denied_degrades_to_unknown() {
    use std::os::unix::fs::PermissionsExt;

    let (_temp, files) = temp_copy_harness("claude-code", "permission-denied");
    let config = files.join("home/.claude/settings.json");
    let restore = RestorePermissions {
        path: config.clone(),
        permissions: std::fs::metadata(&config).unwrap().permissions(),
    };
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o000)).unwrap();

    let output = run_harness_files(&files, &["scan", "--json"]);
    drop(restore);

    assert_eq!(output.status.code(), Some(2));
    let report = json_report(&output, "claude-code/permission-denied");
    assert_json_subset(
        &expected_report_for("claude-code", "permission-denied"),
        &report,
        "claude-code/permission-denied",
    );
    let reason = report["tools"][0]["findings"][0]["unknown_reason"]
        .as_str()
        .unwrap();
    assert!(reason.contains("permission"));
}

#[cfg(unix)]
#[test]
fn claude_code_symlink_config_is_not_followed() {
    let (_temp, files) = temp_copy_harness("claude-code", "symlink-config");
    let claude_home = files.join("home/.claude");
    std::os::unix::fs::symlink(
        claude_home.join("real-settings.json"),
        claude_home.join("settings.json"),
    )
    .unwrap();

    let output = run_harness_files(&files, &["scan", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "refused read degrades the scan"
    );
    let report = json_report(&output, "claude-code/symlink-config");
    assert_json_subset(
        &expected_report_for("claude-code", "symlink-config"),
        &report,
        "claude-code/symlink-config",
    );
    assert_eq!(report["tools"][0]["findings"][0]["status"], "unknown");
    let reason = report["tools"][0]["findings"][0]["unknown_reason"]
        .as_str()
        .unwrap();
    assert!(reason.contains("symlink"));
}

#[test]
fn claude_code_non_utf8_config_degrades_to_unknown() {
    let (_temp, files) = temp_copy_harness("claude-code", "non-utf8");
    let config = files.join("home/.claude/settings.json");
    std::fs::write(&config, [0xff, 0xfe, 0x00, 0x41]).unwrap();

    let output = run_harness_files(&files, &["scan", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let report = json_report(&output, "claude-code/non-utf8");
    assert_json_subset(
        &expected_report_for("claude-code", "non-utf8"),
        &report,
        "claude-code/non-utf8",
    );
    let reason = report["tools"][0]["findings"][0]["unknown_reason"]
        .as_str()
        .unwrap();
    assert!(reason.contains("UTF-8"));
}
