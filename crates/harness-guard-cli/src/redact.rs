//! Output redaction for home-derived paths (§7.3).
//!
//! Usernames must never appear in output, so a path below the resolved home
//! directory is rendered relative to `~`.
use std::path::Path;

pub fn redact_home(path: &str, home: Option<&Path>) -> String {
    let Some(home) = home else {
        return path.to_string();
    };
    let Ok(relative) = Path::new(path).strip_prefix(home) else {
        return path.to_string();
    };

    if relative.as_os_str().is_empty() {
        "~".to_string()
    } else {
        let normalized = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        format!("~/{normalized}")
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
}
