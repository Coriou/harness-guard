//! Bounded, refusing reads (§9): symlink_metadata before open, regular
//! files only, 1 MiB cap, UTF-8 only. Refusal is a value, not an error —
//! callers map it to `unknown` findings.
use crate::discovery::DiscoveryRoot;
use std::io::Read;

pub const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

#[derive(Debug)]
pub enum ConfigReadOutcome {
    /// Home dir or config file absent — tool undetected or unconfigured.
    NoConfig,
    Ok(String),
    Refused(RefusalReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalReason {
    Symlink,
    NotRegularFile,
    Oversized,
    PermissionDenied,
    NotUtf8,
    Io,
}

impl RefusalReason {
    /// Structural, value-free text used in unknown_reason and diagnostics.
    pub fn describe(&self) -> &'static str {
        match self {
            Self::Symlink => "config file is a symlink — not followed",
            Self::NotRegularFile => "config path is not a regular file",
            Self::Oversized => "config file exceeds the 1 MiB parse bound",
            Self::PermissionDenied => "config file is not readable (permission denied)",
            Self::NotUtf8 => "config file is not valid UTF-8",
            Self::Io => "config file could not be read (I/O error)",
        }
    }
}

pub fn read_config(root: &DiscoveryRoot) -> ConfigReadOutcome {
    let path = root.config_path();
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return ConfigReadOutcome::NoConfig;
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            return ConfigReadOutcome::Refused(RefusalReason::PermissionDenied);
        }
        Err(_) => return ConfigReadOutcome::Refused(RefusalReason::Io),
    };

    if meta.file_type().is_symlink() {
        return ConfigReadOutcome::Refused(RefusalReason::Symlink);
    }
    if !meta.file_type().is_file() {
        return ConfigReadOutcome::Refused(RefusalReason::NotRegularFile);
    }
    if meta.len() > MAX_CONFIG_BYTES {
        return ConfigReadOutcome::Refused(RefusalReason::Oversized);
    }

    let file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            return ConfigReadOutcome::Refused(RefusalReason::PermissionDenied);
        }
        Err(_) => return ConfigReadOutcome::Refused(RefusalReason::Io),
    };

    // Belt-and-suspenders: cap the read itself, not just the stat.
    let mut buffer = Vec::with_capacity(meta.len() as usize);
    if file
        .take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut buffer)
        .is_err()
    {
        return ConfigReadOutcome::Refused(RefusalReason::Io);
    }
    if buffer.len() as u64 > MAX_CONFIG_BYTES {
        return ConfigReadOutcome::Refused(RefusalReason::Oversized);
    }

    match String::from_utf8(buffer) {
        Ok(text) => ConfigReadOutcome::Ok(text),
        Err(_) => ConfigReadOutcome::Refused(RefusalReason::NotUtf8),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveryRoot;
    use std::io::Write;

    fn root_with(config: Option<&[u8]>) -> (tempfile::TempDir, DiscoveryRoot) {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("codex-home");
        std::fs::create_dir_all(&home).unwrap();
        if let Some(bytes) = config {
            std::fs::File::create(home.join("config.toml"))
                .unwrap()
                .write_all(bytes)
                .unwrap();
        }
        let root = DiscoveryRoot {
            codex_home: home,
            path_dirs: vec![],
        };
        (dir, root)
    }

    #[test]
    fn missing_home_is_no_config() {
        let dir = tempfile::tempdir().unwrap();
        let root = DiscoveryRoot {
            codex_home: dir.path().join("nope"),
            path_dirs: vec![],
        };
        assert!(matches!(read_config(&root), ConfigReadOutcome::NoConfig));
    }

    #[test]
    fn missing_file_is_no_config() {
        let (_d, root) = root_with(None);
        assert!(matches!(read_config(&root), ConfigReadOutcome::NoConfig));
    }

    #[test]
    fn regular_file_within_bounds_reads_ok() {
        let (_d, root) = root_with(Some(b"[history]\npersistence = \"none\"\n"));
        match read_config(&root) {
            ConfigReadOutcome::Ok(s) => assert!(s.contains("persistence")),
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_config_is_refused_not_followed() {
        let (_d, root) = root_with(None);
        let target = root.codex_home.join("real.toml");
        std::fs::write(&target, "[history]\npersistence = \"none\"\n").unwrap();
        std::os::unix::fs::symlink(&target, root.codex_home.join("config.toml")).unwrap();
        assert!(matches!(
            read_config(&root),
            ConfigReadOutcome::Refused(RefusalReason::Symlink)
        ));
    }

    #[test]
    fn oversized_config_is_refused() {
        let big = vec![b'#'; MAX_CONFIG_BYTES as usize + 1];
        let (_d, root) = root_with(Some(&big));
        assert!(matches!(
            read_config(&root),
            ConfigReadOutcome::Refused(RefusalReason::Oversized)
        ));
    }

    #[test]
    fn non_utf8_is_refused() {
        let (_d, root) = root_with(Some(&[0xff, 0xfe, 0x00, 0x41]));
        assert!(matches!(
            read_config(&root),
            ConfigReadOutcome::Refused(RefusalReason::NotUtf8)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn permission_denied_is_refused() {
        use std::os::unix::fs::PermissionsExt;
        let (_d, root) = root_with(Some(b"x = 1\n"));
        let p = root.codex_home.join("config.toml");
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o000)).unwrap();
        let out = read_config(&root);
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(matches!(
            out,
            ConfigReadOutcome::Refused(RefusalReason::PermissionDenied)
        ));
    }
}
