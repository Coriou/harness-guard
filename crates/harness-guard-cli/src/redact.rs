//! Output redaction for home-derived paths (§7.3).
//!
//! Usernames must never appear in output, so a path below the resolved home
//! directory is rendered relative to `~`.
use std::path::Path;

pub fn redact_home(path: &str, home: Option<&Path>) -> String {
    redact_under(path, home, "~").unwrap_or_else(|| path.to_string())
}

/// Config paths have two safe render roots. Prefer `~` when the explicit
/// Codex home is below HOME; otherwise use a fixed token and never emit the
/// absolute custom `CODEX_HOME` value.
pub fn redact_config_path(path: &str, home: Option<&Path>, codex_home: &Path) -> String {
    let home_redacted = redact_home(path, home);
    if home_redacted != path {
        return home_redacted;
    }

    redact_under(path, Some(codex_home), "$CODEX_HOME")
        .unwrap_or_else(|| "$CODEX_HOME/config.toml".to_string())
}

fn redact_under(path: &str, root: Option<&Path>, token: &str) -> Option<String> {
    let root = root?;
    let relative = Path::new(path).strip_prefix(root).ok()?;

    if relative.as_os_str().is_empty() {
        Some(token.to_string())
    } else {
        let normalized = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        Some(format!("{token}/{normalized}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_and_descendants_are_redacted() {
        let home = Path::new("/synthetic/alice");
        assert_eq!(redact_home("/synthetic/alice", Some(home)), "~");
        assert_eq!(
            redact_home("/synthetic/alice/.codex/config.toml", Some(home)),
            "~/.codex/config.toml"
        );
    }

    #[test]
    fn path_prefix_that_is_not_a_component_is_unchanged() {
        let home = Path::new("/synthetic/alice");
        assert_eq!(
            redact_home("/synthetic/alice-other/config.toml", Some(home)),
            "/synthetic/alice-other/config.toml"
        );
    }

    #[test]
    fn config_outside_home_uses_symbolic_codex_home() {
        assert_eq!(
            redact_config_path(
                "/synthetic/codex-root/config.toml",
                Some(Path::new("/synthetic/home")),
                Path::new("/synthetic/codex-root"),
            ),
            "$CODEX_HOME/config.toml"
        );
    }
}
