//! Execution-free Codex version detection (§9).
//! NEVER runs the tool. npm layouts yield a version; standalone/Homebrew
//! layouts legitimately yield None → stale-ruleset ("version not detected").
use crate::discovery::DiscoveryRoot;
use crate::readfs::{BoundedReadError, read_bounded_regular_with_hook};
use harness_guard_rules::schema::TestedVersion;
use std::path::{Path, PathBuf};

const MAX_SYMLINK_HOPS: usize = 5;
const MAX_PARENT_WALK: usize = 5;
const EXPECTED_PACKAGE: &str = "@openai/codex";
const MAX_PACKAGE_JSON_BYTES: u64 = 64 * 1024;

pub fn detect_codex_version(root: &DiscoveryRoot) -> Option<String> {
    detect_codex_version_with_hook(root, || {})
}

fn detect_codex_version_with_hook(
    root: &DiscoveryRoot,
    after_package_open: impl FnOnce(),
) -> Option<String> {
    let binary = find_codex_entry(root)?;
    let resolved = resolve_bounded(&binary)?;
    let bytes = read_nearest_package_json_with_hook(&resolved, after_package_open)?;
    let text = String::from_utf8(bytes).ok()?;
    let package: serde_json::Value = serde_json::from_str(&text).ok()?;
    if package.get("name").and_then(|name| name.as_str()) != Some(EXPECTED_PACKAGE) {
        return None;
    }
    let version = package.get("version")?.as_str()?;
    parse_version(version)?;
    Some(version.to_string())
}

/// Tool-on-PATH check used for detection confidence and the `list` command.
pub fn binary_on_path(root: &DiscoveryRoot) -> bool {
    find_codex_entry(root)
        .and_then(|entry| resolve_bounded(&entry))
        .is_some()
}

fn find_codex_entry(root: &DiscoveryRoot) -> Option<PathBuf> {
    root.path_dirs.iter().find_map(|directory| {
        let candidate = directory.join("codex");
        std::fs::symlink_metadata(&candidate)
            .is_ok()
            .then_some(candidate)
    })
}

fn resolve_bounded(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    for _ in 0..=MAX_SYMLINK_HOPS {
        let metadata = std::fs::symlink_metadata(&current).ok()?;
        if !metadata.file_type().is_symlink() {
            return metadata.file_type().is_file().then_some(current);
        }
        let target = std::fs::read_link(&current).ok()?;
        current = if target.is_absolute() {
            target
        } else {
            current.parent()?.join(target)
        };
    }
    None
}

fn read_nearest_package_json_with_hook(
    resolved_binary: &Path,
    after_package_open: impl FnOnce(),
) -> Option<Vec<u8>> {
    let mut directory = resolved_binary.parent()?;
    let mut after_package_open = Some(after_package_open);
    for _ in 0..MAX_PARENT_WALK {
        let candidate = directory.join("package.json");
        match read_bounded_regular_with_hook(&candidate, MAX_PACKAGE_JSON_BYTES, || {
            after_package_open
                .take()
                .expect("package open hook runs at most once")();
        }) {
            Ok(bytes) => return Some(bytes),
            Err(BoundedReadError::NotFound) => {
                directory = directory.parent()?;
            }
            Err(_) => return None,
        }
    }
    None
}

/// Strict X.Y.Z only. Suffixed npm platform-package versions deliberately fail.
pub fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let major = parse_numeric_part(parts.next()?)?;
    let minor = parse_numeric_part(parts.next()?)?;
    let patch = parse_numeric_part(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn parse_numeric_part(part: &str) -> Option<u64> {
    (!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| part.parse().ok())
        .flatten()
}

/// A version is verified iff some tested-version entry matches. `min` may
/// carry the MDN-style `<=` prefix, meaning the range is unbounded below.
pub fn version_in_range(detected: &str, ranges: &[TestedVersion]) -> bool {
    let Some(detected) = parse_version(detected) else {
        return false;
    };

    ranges.iter().any(|range| {
        let Some(maximum) = parse_version(&range.max) else {
            return false;
        };
        if detected > maximum {
            return false;
        }
        match range.min.strip_prefix("<=") {
            Some(_) => true,
            None => parse_version(&range.min)
                .map(|minimum| detected >= minimum)
                .unwrap_or(false),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveryRoot;
    use harness_guard_rules::schema::TestedVersion;

    fn tv(min: &str, max: &str) -> TestedVersion {
        TestedVersion {
            min: min.into(),
            max: max.into(),
            verified_on: "2026-07-14".into(),
        }
    }

    #[test]
    fn parse_strict_triples_only() {
        assert_eq!(parse_version("0.144.4"), Some((0, 144, 4)));
        assert_eq!(parse_version("0.144.4-darwin-arm64"), None);
        assert_eq!(parse_version("v0.144.4"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn le_prefixed_min_is_unbounded_below() {
        let ranges = [tv("<=0.144.4", "0.144.4")];
        assert!(version_in_range("0.1.0", &ranges));
        assert!(version_in_range("0.144.4", &ranges));
        assert!(!version_in_range("0.144.5", &ranges));
        assert!(!version_in_range("9.9.9", &ranges));
    }

    #[test]
    fn plain_min_is_a_real_lower_bound() {
        let ranges = [tv("0.100.0", "0.144.4")];
        assert!(!version_in_range("0.99.9", &ranges));
        assert!(version_in_range("0.100.0", &ranges));
    }

    #[test]
    fn unparseable_detected_version_never_matches() {
        let ranges = [tv("<=0.144.4", "0.144.4")];
        assert!(!version_in_range("0.144.4-darwin-arm64", &ranges));
    }

    fn npm_layout(version_json: &str) -> (tempfile::TempDir, DiscoveryRoot) {
        let dir = tempfile::tempdir().unwrap();
        let package = dir.path().join("node_modules/@openai/codex");
        std::fs::create_dir_all(package.join("bin")).unwrap();
        std::fs::write(package.join("bin/codex"), "#!/usr/bin/env node\n").unwrap();
        std::fs::write(package.join("package.json"), version_json).unwrap();
        let root = DiscoveryRoot {
            codex_home: dir.path().join("codex-home"),
            path_dirs: vec![package.join("bin")],
        };
        (dir, root)
    }

    #[test]
    fn npm_layout_detects_clean_version() {
        let (_dir, root) = npm_layout(r#"{"name": "@openai/codex", "version": "0.144.4"}"#);
        assert_eq!(detect_codex_version(&root), Some("0.144.4".to_string()));
    }

    #[test]
    fn wrong_package_name_is_ignored() {
        let (_dir, root) = npm_layout(r#"{"name": "something-else", "version": "0.144.4"}"#);
        assert_eq!(detect_codex_version(&root), None);
    }

    #[test]
    fn suffixed_version_is_rejected() {
        let (_dir, root) =
            npm_layout(r#"{"name": "@openai/codex", "version": "0.144.4-darwin-arm64"}"#);
        assert_eq!(detect_codex_version(&root), None);
    }

    #[test]
    fn no_package_json_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join("codex"), "binary").unwrap();
        let root = DiscoveryRoot {
            codex_home: dir.path().join("x"),
            path_dirs: vec![bin],
        };
        assert_eq!(detect_codex_version(&root), None);
    }

    #[test]
    fn oversized_package_json_is_refused() {
        let padding = "x".repeat(MAX_PACKAGE_JSON_BYTES as usize);
        let package =
            format!(r#"{{"name":"@openai/codex","version":"0.144.4","padding":"{padding}"}}"#);
        let (_dir, root) = npm_layout(&package);
        assert_eq!(detect_codex_version(&root), None);
    }

    #[test]
    fn codex_directory_is_not_a_binary() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(bin.join("codex")).unwrap();
        std::fs::write(
            bin.join("package.json"),
            r#"{"name":"@openai/codex","version":"0.144.4"}"#,
        )
        .unwrap();
        let root = DiscoveryRoot {
            codex_home: dir.path().join("x"),
            path_dirs: vec![bin],
        };
        assert!(!binary_on_path(&root));
        assert_eq!(detect_codex_version(&root), None);
    }

    #[test]
    fn regular_package_replacement_after_open_uses_stable_original_handle() {
        let (_dir, root) = npm_layout(r#"{"name":"@openai/codex","version":"0.144.4"}"#);
        let package_json = root.path_dirs[0].parent().unwrap().join("package.json");
        let displaced = package_json.with_file_name("original.json");
        let replacement = package_json.with_file_name("replacement.json");
        std::fs::write(
            &replacement,
            r#"{"name":"@openai/codex","version":"9.9.9","secret":"must-not-be-read"}"#,
        )
        .unwrap();

        let detected = detect_codex_version_with_hook(&root, || {
            std::fs::rename(&package_json, &displaced).unwrap();
            std::fs::rename(&replacement, &package_json).unwrap();
        });

        assert_eq!(detected, Some("0.144.4".to_string()));
        assert_ne!(detected, Some("9.9.9".to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_chain_is_resolved_with_bounded_hops() {
        let dir = tempfile::tempdir().unwrap();
        let package = dir.path().join("lib/node_modules/@openai/codex");
        std::fs::create_dir_all(package.join("bin")).unwrap();
        std::fs::write(package.join("bin/codex"), "shim").unwrap();
        std::fs::write(
            package.join("package.json"),
            r#"{"name": "@openai/codex", "version": "0.144.4"}"#,
        )
        .unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::os::unix::fs::symlink(package.join("bin/codex"), bin.join("codex")).unwrap();
        let root = DiscoveryRoot {
            codex_home: dir.path().join("x"),
            path_dirs: vec![bin],
        };
        assert_eq!(detect_codex_version(&root), Some("0.144.4".to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_loop_terminates_with_none() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::os::unix::fs::symlink(bin.join("b"), bin.join("codex")).unwrap();
        std::os::unix::fs::symlink(bin.join("codex"), bin.join("b")).unwrap();
        let root = DiscoveryRoot {
            codex_home: dir.path().join("x"),
            path_dirs: vec![bin],
        };
        assert_eq!(detect_codex_version(&root), None);
    }
}
