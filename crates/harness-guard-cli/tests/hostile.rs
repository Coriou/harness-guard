mod common;

use common::*;
use std::path::{Path, PathBuf};

/// Copy a fixture's files into a tempdir so runtime mutation never touches
/// the committed tree (and absolutely never the real ~/.codex).
fn temp_copy(case: &str) -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("files");
    copy_dir(&fixture(case), &destination);
    (temp, destination)
}

fn copy_dir(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination_entry = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &destination_entry);
        } else {
            std::fs::copy(entry.path(), destination_entry).unwrap();
        }
    }
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
        stderr.contains("~/codex-home/config.toml"),
        "diagnostic should name only the redacted config path: {stderr}"
    );
    assert!(
        !output.stdout.is_empty(),
        "degraded scan must still render a report"
    );
}
