//! The closed harness set (§3) and per-harness descriptor facts (§5.1).
//! Descriptors are code, not config — adding a harness is a deliberate,
//! compile-visible act; every match on HarnessId is exhaustive. Descriptor
//! facts must be traceable to the evidence recorded with that harness's
//! rules; entries still awaiting fresh retrieval are None and say so.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HarnessId {
    ClaudeCode,
    Codex,
    GrokBuild,
}

impl HarnessId {
    /// Alphabetical by tool id — the contractual report/list ordering (§5.5).
    pub const ALL: [HarnessId; 3] = [
        HarnessId::ClaudeCode,
        HarnessId::Codex,
        HarnessId::GrokBuild,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            HarnessId::ClaudeCode => "claude-code",
            HarnessId::Codex => "codex",
            HarnessId::GrokBuild => "grok-build",
        }
    }

    pub fn parse(text: &str) -> Option<HarnessId> {
        match text {
            "claude-code" => Some(HarnessId::ClaudeCode),
            "codex" => Some(HarnessId::Codex),
            "grok-build" => Some(HarnessId::GrokBuild),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Json,
}

pub struct HarnessDescriptor {
    pub id: HarnessId,
    /// User-scope config file name inside the harness home (§5.1 table).
    pub config_file: &'static str,
    pub config_format: ConfigFormat,
    /// PATH entry name used for detection. None disables PATH-based
    /// detection for that harness.
    pub path_binary: Option<&'static str>,
    /// npm package the version walk expects. None skips the npm walk; a
    /// harness may still detect version via a non-npm fallback (e.g. Grok
    /// managed-install symlink basename parsing).
    pub npm_package: Option<&'static str>,
    /// Symbolic token for a redacted config path that is not under the user
    /// home (reachable only via a home-override env var in the CLI crate).
    pub home_token: &'static str,
}

static CLAUDE_CODE: HarnessDescriptor = HarnessDescriptor {
    id: HarnessId::ClaudeCode,
    config_file: "settings.json",
    config_format: ConfigFormat::Json,
    path_binary: Some("claude"),
    npm_package: Some("@anthropic-ai/claude-code"),
    // Token only; whether a home-override env var exists is a CLI-crate
    // fresh-retrieval item (§5.1, lead: CLAUDE_CONFIG_DIR) — see Task 15.
    home_token: "$CLAUDE_HOME",
};

static CODEX: HarnessDescriptor = HarnessDescriptor {
    id: HarnessId::Codex,
    config_file: "config.toml",
    config_format: ConfigFormat::Toml,
    path_binary: Some("codex"),
    npm_package: Some("@openai/codex"),
    home_token: "$CODEX_HOME",
};

static GROK_BUILD: HarnessDescriptor = HarnessDescriptor {
    id: HarnessId::GrokBuild,
    config_file: "config.toml",
    config_format: ConfigFormat::Toml,
    // Evidence pack docs/research/evidence/grok-build/2026-07-17 (SOURCE_REV
    // 124d85bc5dc6e7805560215fcc6d5413944920e1): PATH binary `grok`, npm
    // package `@xai-official/grok` still published; managed installs expose
    // version via symlink target basename (version.rs fallback).
    path_binary: Some("grok"),
    npm_package: Some("@xai-official/grok"),
    home_token: "$GROK_HOME",
};

pub fn descriptor(id: HarnessId) -> &'static HarnessDescriptor {
    match id {
        HarnessId::ClaudeCode => &CLAUDE_CODE,
        HarnessId::Codex => &CODEX,
        HarnessId::GrokBuild => &GROK_BUILD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_and_all_is_alphabetical() {
        for id in HarnessId::ALL {
            assert_eq!(HarnessId::parse(id.as_str()), Some(id));
        }
        let mut names: Vec<&str> = HarnessId::ALL.iter().map(|id| id.as_str()).collect();
        let sorted = {
            let mut s = names.clone();
            s.sort_unstable();
            s
        };
        assert_eq!(names, sorted, "ALL must stay alphabetical by tool id");
        names.dedup();
        assert_eq!(names.len(), 3);
        assert_eq!(HarnessId::parse("cursor"), None);
    }

    #[test]
    fn descriptors_carry_the_spec_table_facts() {
        assert_eq!(descriptor(HarnessId::Codex).config_file, "config.toml");
        assert_eq!(
            descriptor(HarnessId::Codex).npm_package,
            Some("@openai/codex")
        );
        assert_eq!(
            descriptor(HarnessId::ClaudeCode).config_file,
            "settings.json"
        );
        assert_eq!(
            descriptor(HarnessId::ClaudeCode).config_format,
            ConfigFormat::Json
        );
        assert_eq!(
            descriptor(HarnessId::ClaudeCode).npm_package,
            Some("@anthropic-ai/claude-code")
        );
        // Grok detection from 2026-07-17 evidence pack (SOURCE_REV 124d85bc…).
        assert_eq!(descriptor(HarnessId::GrokBuild).path_binary, Some("grok"));
        assert_eq!(
            descriptor(HarnessId::GrokBuild).npm_package,
            Some("@xai-official/grok")
        );
        assert_eq!(descriptor(HarnessId::GrokBuild).config_file, "config.toml");
        assert_eq!(
            descriptor(HarnessId::GrokBuild).config_format,
            ConfigFormat::Toml
        );
    }
}
