//! Bounded, refusing reads (§9): hardened no-follow/nonblocking open,
//! opened-handle validation, regular files only, 1 MiB cap, UTF-8 only.
//! Refusal is a value, not an error — callers map it to `unknown` findings.
use crate::discovery::DiscoveryRoot;
use crate::harness::HarnessId;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path};

pub const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathProbe {
    Missing,
    Present,
    Refused,
}

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
            Self::Symlink => "config path contains a symlink or reparse point — not followed",
            Self::NotRegularFile => "config path is not a regular file",
            Self::Oversized => "config file exceeds the 1 MiB parse bound",
            Self::PermissionDenied => "config file is not readable (permission denied)",
            Self::NotUtf8 => "config file is not valid UTF-8",
            Self::Io => "config file could not be read (I/O error)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundedReadError {
    NotFound,
    Symlink,
    NotRegularFile,
    Oversized,
    PermissionDenied,
    Io,
}

pub fn read_config(root: &DiscoveryRoot, harness: HarnessId) -> ConfigReadOutcome {
    let path = root.config_path(harness);
    let bytes = match read_bounded_regular(&path, MAX_CONFIG_BYTES) {
        Ok(bytes) => bytes,
        Err(BoundedReadError::NotFound) => return ConfigReadOutcome::NoConfig,
        Err(BoundedReadError::Symlink) => {
            return ConfigReadOutcome::Refused(RefusalReason::Symlink);
        }
        Err(BoundedReadError::NotRegularFile) => {
            return ConfigReadOutcome::Refused(RefusalReason::NotRegularFile);
        }
        Err(BoundedReadError::Oversized) => {
            return ConfigReadOutcome::Refused(RefusalReason::Oversized);
        }
        Err(BoundedReadError::PermissionDenied) => {
            return ConfigReadOutcome::Refused(RefusalReason::PermissionDenied);
        }
        Err(BoundedReadError::Io) => {
            return ConfigReadOutcome::Refused(RefusalReason::Io);
        }
    };

    match String::from_utf8(bytes) {
        Ok(text) => ConfigReadOutcome::Ok(text),
        Err(_) => ConfigReadOutcome::Refused(RefusalReason::NotUtf8),
    }
}

/// Probe a directory without following any component. A refusal still counts
/// as discovery evidence, but callers must not treat it as safely readable.
pub fn probe_directory(path: &Path) -> PathProbe {
    match open_directory_no_follow(path) {
        Ok(_) => PathProbe::Present,
        Err(BoundedReadError::NotFound) => PathProbe::Missing,
        Err(_) => PathProbe::Refused,
    }
}

/// Probe a regular file through the same component-by-component traversal used
/// for bounded reads, without reading its contents.
pub fn probe_regular_file(path: &Path) -> PathProbe {
    match open_read_only_no_follow_nonblocking(path, || {}) {
        Ok(file) => match file.metadata() {
            Ok(metadata) if metadata.file_type().is_file() => PathProbe::Present,
            Ok(_) | Err(_) => PathProbe::Refused,
        },
        Err(BoundedReadError::NotFound) => PathProbe::Missing,
        Err(_) => PathProbe::Refused,
    }
}

/// Read through one hardened handle. Type, size, and content decisions all
/// come from that handle, so path replacement after open cannot redirect the
/// read to replacement content.
pub(crate) fn read_bounded_regular(
    path: &Path,
    max_bytes: u64,
) -> Result<Vec<u8>, BoundedReadError> {
    read_bounded_regular_with_hook(path, max_bytes, || {})
}

pub(crate) fn read_bounded_regular_with_hook(
    path: &Path,
    max_bytes: u64,
    after_open: impl FnOnce(),
) -> Result<Vec<u8>, BoundedReadError> {
    read_bounded_regular_with_hooks(path, max_bytes, || {}, after_open)
}

fn read_bounded_regular_with_hooks(
    path: &Path,
    max_bytes: u64,
    before_final_open: impl FnOnce(),
    after_open: impl FnOnce(),
) -> Result<Vec<u8>, BoundedReadError> {
    let file = open_read_only_no_follow_nonblocking(path, before_final_open)?;
    let opened = file.metadata().map_err(classify_io_error)?;
    if opened.file_type().is_symlink() {
        return Err(BoundedReadError::Symlink);
    }
    if !opened.file_type().is_file() {
        return Err(BoundedReadError::NotRegularFile);
    }
    if opened.len() > max_bytes {
        return Err(BoundedReadError::Oversized);
    }

    after_open();

    let mut bytes = Vec::with_capacity(opened.len() as usize);
    file.take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(classify_io_error)?;
    if bytes.len() as u64 > max_bytes {
        return Err(BoundedReadError::Oversized);
    }
    Ok(bytes)
}

fn classify_io_error(error: std::io::Error) -> BoundedReadError {
    match error.kind() {
        std::io::ErrorKind::NotFound => BoundedReadError::NotFound,
        std::io::ErrorKind::PermissionDenied => BoundedReadError::PermissionDenied,
        _ => BoundedReadError::Io,
    }
}

#[cfg(unix)]
fn open_read_only_no_follow_nonblocking(
    path: &Path,
    before_final_open: impl FnOnce(),
) -> Result<File, BoundedReadError> {
    use rustix::fs::{AtFlags, FileType, Mode, OFlags, open, openat, statat};

    let directory_flags =
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let file_flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let start = if path.is_absolute() { "/" } else { "." };
    let mut directory = open(start, directory_flags, Mode::empty())
        .map_err(|error| classify_io_error(error.into()))?;
    let mut components = path
        .components()
        .filter(|component| !matches!(component, Component::RootDir | Component::CurDir))
        .peekable();
    let mut before_final_open = Some(before_final_open);

    while let Some(component) = components.next() {
        let name = component.as_os_str();
        if statat(&directory, name, AtFlags::SYMLINK_NOFOLLOW)
            .is_ok_and(|metadata| FileType::from_raw_mode(metadata.st_mode).is_symlink())
        {
            return Err(BoundedReadError::Symlink);
        }

        if components.peek().is_none() {
            before_final_open
                .take()
                .expect("final-open hook runs exactly once")();
            return openat(&directory, name, file_flags, Mode::empty())
                .map(File::from)
                .map_err(|error| classify_unix_open_error(&directory, name, error));
        }

        directory = openat(&directory, name, directory_flags, Mode::empty())
            .map_err(|error| classify_unix_open_error(&directory, name, error))?;
    }

    Err(BoundedReadError::Io)
}

#[cfg(unix)]
fn open_directory_no_follow(path: &Path) -> Result<File, BoundedReadError> {
    use rustix::fs::{Mode, OFlags, open, openat};

    let directory_flags =
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let start = if path.is_absolute() { "/" } else { "." };
    let mut directory = open(start, directory_flags, Mode::empty())
        .map_err(|error| classify_io_error(error.into()))?;

    for component in path
        .components()
        .filter(|component| !matches!(component, Component::RootDir | Component::CurDir))
    {
        let name = component.as_os_str();
        directory = openat(&directory, name, directory_flags, Mode::empty())
            .map_err(|error| classify_unix_open_error(&directory, name, error))?;
    }

    Ok(File::from(directory))
}

#[cfg(unix)]
fn classify_unix_open_error(
    directory: &impl std::os::fd::AsFd,
    name: &std::ffi::OsStr,
    error: rustix::io::Errno,
) -> BoundedReadError {
    use rustix::fs::{AtFlags, FileType, statat};

    if statat(directory, name, AtFlags::SYMLINK_NOFOLLOW)
        .is_ok_and(|metadata| FileType::from_raw_mode(metadata.st_mode).is_symlink())
    {
        BoundedReadError::Symlink
    } else {
        classify_io_error(error.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveryRoot;
    use crate::harness::HarnessId;
    use std::io::Write;

    fn root_with(config: Option<&[u8]>) -> (tempfile::TempDir, DiscoveryRoot) {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let home = base.join("codex-home");
        std::fs::create_dir_all(&home).unwrap();
        if let Some(bytes) = config {
            std::fs::File::create(home.join("config.toml"))
                .unwrap()
                .write_all(bytes)
                .unwrap();
        }
        let root = DiscoveryRoot {
            codex_home: home,
            claude_home: base.join("absent-claude-home"),
            grok_home: base.join("absent-grok-home"),
            path_dirs: vec![],
        };
        (dir, root)
    }

    #[test]
    fn missing_home_is_no_config() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let root = DiscoveryRoot {
            codex_home: base.join("nope"),
            claude_home: base.join("absent-claude-home"),
            grok_home: base.join("absent-grok-home"),
            path_dirs: vec![],
        };
        assert!(matches!(
            read_config(&root, HarnessId::Codex),
            ConfigReadOutcome::NoConfig
        ));
    }

    #[test]
    fn missing_file_is_no_config() {
        let (_d, root) = root_with(None);
        assert!(matches!(
            read_config(&root, HarnessId::Codex),
            ConfigReadOutcome::NoConfig
        ));
    }

    #[test]
    fn safe_probes_distinguish_present_and_missing_paths() {
        let (_dir, root) = root_with(Some(b"[history]\npersistence = \"none\"\n"));
        assert_eq!(probe_directory(&root.codex_home), PathProbe::Present);
        assert_eq!(
            probe_regular_file(&root.config_path(HarnessId::Codex)),
            PathProbe::Present
        );
        assert_eq!(
            probe_regular_file(&root.codex_home.join("missing.toml")),
            PathProbe::Missing
        );
    }

    #[test]
    fn regular_file_within_bounds_reads_ok() {
        let (_d, root) = root_with(Some(b"[history]\npersistence = \"none\"\n"));
        match read_config(&root, HarnessId::Codex) {
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
            read_config(&root, HarnessId::Codex),
            ConfigReadOutcome::Refused(RefusalReason::Symlink)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_codex_home_is_refused_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let real_home = base.join("real-codex-home");
        std::fs::create_dir(&real_home).unwrap();
        std::fs::write(
            real_home.join("config.toml"),
            "[history]\npersistence = \"none\"\n",
        )
        .unwrap();
        let linked_home = base.join("codex-home");
        std::os::unix::fs::symlink(&real_home, &linked_home).unwrap();
        let root = DiscoveryRoot {
            codex_home: linked_home,
            claude_home: base.join("absent-claude-home"),
            grok_home: base.join("absent-grok-home"),
            path_dirs: vec![],
        };

        assert!(matches!(
            read_config(&root, HarnessId::Codex),
            ConfigReadOutcome::Refused(RefusalReason::Symlink)
        ));
        assert_eq!(probe_directory(&root.codex_home), PathProbe::Refused);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_codex_home_ancestor_is_refused_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let real_parent = base.join("real-parent");
        let real_home = real_parent.join("codex-home");
        std::fs::create_dir_all(&real_home).unwrap();
        std::fs::write(
            real_home.join("config.toml"),
            "[history]\npersistence = \"none\"\n",
        )
        .unwrap();
        let linked_parent = base.join("linked-parent");
        std::os::unix::fs::symlink(&real_parent, &linked_parent).unwrap();
        let root = DiscoveryRoot {
            codex_home: linked_parent.join("codex-home"),
            claude_home: base.join("absent-claude-home"),
            grok_home: base.join("absent-grok-home"),
            path_dirs: vec![],
        };

        assert!(matches!(
            read_config(&root, HarnessId::Codex),
            ConfigReadOutcome::Refused(RefusalReason::Symlink)
        ));
    }

    #[test]
    fn regular_config_replacement_after_open_reads_stable_original_handle() {
        let (_dir, root) = root_with(Some(b"[history]\npersistence = \"none\"\n"));
        let config = root.config_path(HarnessId::Codex);
        let displaced = root.codex_home.join("original.toml");
        let replacement = root.codex_home.join("replacement.toml");
        std::fs::write(
            &replacement,
            "[history]\npersistence = \"replacement-must-not-be-read\"\n",
        )
        .unwrap();

        let outcome = read_bounded_regular_with_hook(&config, MAX_CONFIG_BYTES, || {
            std::fs::rename(&config, &displaced).unwrap();
            std::fs::rename(&replacement, &config).unwrap();
        });

        let text = String::from_utf8(outcome.unwrap()).unwrap();
        assert!(text.contains("persistence = \"none\""));
        assert!(!text.contains("replacement-must-not-be-read"));
    }

    #[cfg(unix)]
    #[test]
    fn ancestor_replacement_before_final_open_cannot_redirect_read() {
        let (_dir, root) = root_with(Some(b"[history]\npersistence = \"none\"\n"));
        let config = root.config_path(HarnessId::Codex);
        let displaced_home = root.codex_home.with_file_name("original-codex-home");
        let external_home = root.codex_home.with_file_name("external-codex-home");
        std::fs::create_dir(&external_home).unwrap();
        std::fs::write(
            external_home.join("config.toml"),
            "[history]\npersistence = \"redirected-must-not-be-read\"\n",
        )
        .unwrap();

        let outcome = read_bounded_regular_with_hooks(
            &config,
            MAX_CONFIG_BYTES,
            || {
                std::fs::rename(&root.codex_home, &displaced_home).unwrap();
                std::os::unix::fs::symlink(&external_home, &root.codex_home).unwrap();
            },
            || {},
        );

        let text = String::from_utf8(outcome.unwrap()).unwrap();
        assert!(text.contains("persistence = \"none\""));
        assert!(!text.contains("redirected-must-not-be-read"));
    }

    #[test]
    fn oversized_config_is_refused() {
        let big = vec![b'#'; MAX_CONFIG_BYTES as usize + 1];
        let (_d, root) = root_with(Some(&big));
        assert!(matches!(
            read_config(&root, HarnessId::Codex),
            ConfigReadOutcome::Refused(RefusalReason::Oversized)
        ));
    }

    #[test]
    fn non_utf8_is_refused() {
        let (_d, root) = root_with(Some(&[0xff, 0xfe, 0x00, 0x41]));
        assert!(matches!(
            read_config(&root, HarnessId::Codex),
            ConfigReadOutcome::Refused(RefusalReason::NotUtf8)
        ));
    }

    #[test]
    fn directory_at_config_path_is_refused_as_non_regular() {
        let (_dir, root) = root_with(None);
        std::fs::create_dir(root.config_path(HarnessId::Codex)).unwrap();
        assert!(matches!(
            read_config(&root, HarnessId::Codex),
            ConfigReadOutcome::Refused(RefusalReason::NotRegularFile)
        ));
        assert_eq!(
            probe_regular_file(&root.config_path(HarnessId::Codex)),
            PathProbe::Refused
        );
    }

    #[cfg(unix)]
    #[test]
    fn socket_at_config_path_is_refused_without_blocking() {
        let (_dir, root) = root_with(None);
        let _listener =
            std::os::unix::net::UnixListener::bind(root.config_path(HarnessId::Codex)).unwrap();
        assert!(matches!(
            read_config(&root, HarnessId::Codex),
            ConfigReadOutcome::Refused(_)
        ));
        assert_eq!(
            probe_regular_file(&root.config_path(HarnessId::Codex)),
            PathProbe::Refused
        );
    }

    #[cfg(unix)]
    #[test]
    fn permission_denied_is_refused() {
        use std::os::unix::fs::PermissionsExt;
        let (_d, root) = root_with(Some(b"x = 1\n"));
        let p = root.codex_home.join("config.toml");
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o000)).unwrap();
        let out = read_config(&root, HarnessId::Codex);
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(matches!(
            out,
            ConfigReadOutcome::Refused(RefusalReason::PermissionDenied)
        ));
    }
}
