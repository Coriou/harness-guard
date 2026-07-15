//! Injected roots — the ONLY way core learns about the filesystem (§9).
use std::path::PathBuf;

/// Explicit discovery scope. Only the CLI crate constructs this from the
/// real environment; tests always pass fixture paths. Core has no other
/// door to the filesystem's ambient state (clippy-enforced).
#[derive(Debug, Clone)]
pub struct DiscoveryRoot {
    pub codex_home: PathBuf,
    pub path_dirs: Vec<PathBuf>,
}

impl DiscoveryRoot {
    pub fn config_path(&self) -> PathBuf {
        self.codex_home.join("config.toml")
    }

    /// Tool detection: the Codex home exists, or a Codex entry sits on PATH.
    pub fn codex_home_exists(&self) -> bool {
        self.codex_home.is_dir()
    }
}
