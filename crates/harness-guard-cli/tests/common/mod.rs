use std::path::{Path, PathBuf};
use std::process::Output;

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

pub fn fixture(case: &str) -> PathBuf {
    repo_root().join("fixtures/codex").join(case).join("files")
}

/// Run the binary with every ambient variable cleared. `CODEX_HOME`, `PATH`,
/// and `HOME` point only into a synthetic fixture, so the real home and Codex
/// config are unreachable by construction.
pub fn run_in(files_root: &Path, args: &[&str]) -> Output {
    run_with_roots(
        &files_root.join("codex-home"),
        &files_root.join("path"),
        files_root,
        args,
    )
}

/// Run against synthetic roots with the environment cleared but deliberately
/// omit `NO_COLOR`, so explicit `--color` behavior can be tested independently.
#[allow(dead_code)] // Used by scan_snapshots; integration tests compile this module separately.
pub fn run_in_without_no_color(files_root: &Path, args: &[&str]) -> Output {
    run_with_roots_and_color_env(
        &files_root.join("codex-home"),
        &files_root.join("path"),
        files_root,
        args,
        false,
    )
}

pub fn run_with_roots(codex_home: &Path, path_dir: &Path, home: &Path, args: &[&str]) -> Output {
    run_with_roots_and_color_env(codex_home, path_dir, home, args, true)
}

pub fn run_case(case: &str, args: &[&str]) -> Output {
    run_in(&fixture(case), args)
}

fn run_with_roots_and_color_env(
    codex_home: &Path,
    path_dir: &Path,
    home: &Path,
    args: &[&str],
    set_no_color: bool,
) -> Output {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_harness-guard"));
    command
        .args(args)
        .env_clear()
        .env("CODEX_HOME", codex_home)
        .env("PATH", path_dir)
        .env("HOME", home);
    if set_no_color {
        command.env("NO_COLOR", "1");
    }
    command.output().expect("harness-guard binary runs")
}

/// Recursively assert that every expected key exists and matches. Arrays are
/// ordered and match element-by-element because report ordering is contractual.
#[allow(dead_code)] // Only fixture-golden test crates need the subset assertion.
pub fn assert_json_subset(expected: &serde_json::Value, actual: &serde_json::Value, path: &str) {
    use serde_json::Value;

    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            for (key, expected_value) in expected {
                let actual_value = actual
                    .get(key)
                    .unwrap_or_else(|| panic!("missing key {path}.{key}"));
                assert_json_subset(expected_value, actual_value, &format!("{path}.{key}"));
            }
        }
        (Value::Array(expected), Value::Array(actual)) => {
            assert_eq!(
                expected.len(),
                actual.len(),
                "array length mismatch at {path}"
            );
            for (index, (expected_value, actual_value)) in expected.iter().zip(actual).enumerate() {
                assert_json_subset(expected_value, actual_value, &format!("{path}[{index}]"));
            }
        }
        _ => assert_eq!(expected, actual, "value mismatch at {path}"),
    }
}
