//! Injected roots — the ONLY way core learns about the filesystem (§9).
use crate::harness::{HarnessId, descriptor};
use std::path::{Path, PathBuf};

/// Explicit discovery scope: one explicit home per harness (§5.1). Only the
/// CLI crate constructs this from the real environment; tests always pass
/// fixture paths. Core has no other door to ambient state (clippy-enforced).
#[derive(Debug, Clone)]
pub struct DiscoveryRoot {
    pub codex_home: PathBuf,
    pub claude_home: PathBuf,
    pub grok_home: PathBuf,
    pub path_dirs: Vec<PathBuf>,
}

impl DiscoveryRoot {
    pub fn home(&self, harness: HarnessId) -> &Path {
        match harness {
            HarnessId::ClaudeCode => &self.claude_home,
            HarnessId::Codex => &self.codex_home,
            HarnessId::GrokBuild => &self.grok_home,
        }
    }

    pub fn config_path(&self, harness: HarnessId) -> PathBuf {
        self.home(harness).join(descriptor(harness).config_file)
    }
}
