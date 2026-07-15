//! Bounded, refusing reads (§9): symlink_metadata before open, regular
//! files only, 1 MiB cap, UTF-8 only. Refusal is a value, not an error —
//! callers map it to `unknown` findings.
use crate::discovery::DiscoveryRoot;
use std::fs::{File, Metadata, OpenOptions};
use std::io::Read;
use std::path::Path;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundedReadError {
    NotFound,
    Symlink,
    NotRegularFile,
    Oversized,
    PermissionDenied,
    ChangedBeforeOpen,
    Io,
}

pub fn read_config(root: &DiscoveryRoot) -> ConfigReadOutcome {
    let path = root.config_path();
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
        Err(BoundedReadError::ChangedBeforeOpen | BoundedReadError::Io) => {
            return ConfigReadOutcome::Refused(RefusalReason::Io);
        }
    };

    match String::from_utf8(bytes) {
        Ok(text) => ConfigReadOutcome::Ok(text),
        Err(_) => ConfigReadOutcome::Refused(RefusalReason::NotUtf8),
    }
}

/// Read a bounded regular file without following a path swapped to a symlink
/// between metadata inspection and open. The opened handle is revalidated
/// before any bytes are read.
pub(crate) fn read_bounded_regular(
    path: &Path,
    max_bytes: u64,
) -> Result<Vec<u8>, BoundedReadError> {
    read_bounded_regular_with_hook(path, max_bytes, || {})
}

pub(crate) fn read_bounded_regular_with_hook(
    path: &Path,
    max_bytes: u64,
    after_metadata: impl FnOnce(),
) -> Result<Vec<u8>, BoundedReadError> {
    let before = std::fs::symlink_metadata(path).map_err(classify_io_error)?;
    if before.file_type().is_symlink() {
        return Err(BoundedReadError::Symlink);
    }
    if !before.file_type().is_file() {
        return Err(BoundedReadError::NotRegularFile);
    }
    if before.len() > max_bytes {
        return Err(BoundedReadError::Oversized);
    }

    after_metadata();

    let file = open_read_only_no_follow_nonblocking(path).map_err(classify_io_error)?;
    let opened = file.metadata().map_err(classify_io_error)?;
    if !opened.file_type().is_file() {
        return Err(BoundedReadError::NotRegularFile);
    }
    if !same_file_identity(&before, &opened) {
        return Err(BoundedReadError::ChangedBeforeOpen);
    }
    if opened.len() > max_bytes {
        return Err(BoundedReadError::Oversized);
    }

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

fn open_read_only_no_follow_nonblocking(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    configure_hardened_open(&mut options);
    options.open(path)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn configure_hardened_open(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    const O_NONBLOCK: i32 = 0x800;
    const O_NOFOLLOW: i32 = 0x20_000;
    options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
}

#[cfg(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
))]
fn configure_hardened_open(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    const O_NONBLOCK: i32 = 0x4;
    const O_NOFOLLOW: i32 = 0x100;
    options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
}

#[cfg(any(target_os = "solaris", target_os = "illumos"))]
fn configure_hardened_open(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    const O_NONBLOCK: i32 = 0x80;
    const O_NOFOLLOW: i32 = 0x20_000;
    options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
}

#[cfg(windows)]
fn configure_hardened_open(options: &mut OpenOptions) {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly",
    target_os = "solaris",
    target_os = "illumos",
    windows
)))]
fn configure_hardened_open(_options: &mut OpenOptions) {}

#[cfg(unix)]
fn same_file_identity(before: &Metadata, opened: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    before.dev() == opened.dev() && before.ino() == opened.ino()
}

#[cfg(not(unix))]
fn same_file_identity(_before: &Metadata, _opened: &Metadata) -> bool {
    true
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

    #[cfg(unix)]
    #[test]
    fn config_swap_between_metadata_and_open_is_refused_without_replacement_content() {
        let (_dir, root) = root_with(Some(b"[history]\npersistence = \"none\"\n"));
        let config = root.config_path();
        let replacement = root.codex_home.join("replacement.toml");
        std::fs::write(
            &replacement,
            "[history]\npersistence = \"replacement-must-not-be-read\"\n",
        )
        .unwrap();

        let outcome = read_bounded_regular_with_hook(&config, MAX_CONFIG_BYTES, || {
            std::fs::remove_file(&config).unwrap();
            std::os::unix::fs::symlink(&replacement, &config).unwrap();
        });

        assert!(outcome.is_err());
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
