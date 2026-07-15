//! Bounded, refusing reads (§9): hardened no-follow/nonblocking open,
//! opened-handle validation, regular files only, 1 MiB cap, UTF-8 only.
//! Refusal is a value, not an error — callers map it to `unknown` findings.
use crate::discovery::DiscoveryRoot;
use std::fs::{File, OpenOptions};
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
        Err(BoundedReadError::Io) => {
            return ConfigReadOutcome::Refused(RefusalReason::Io);
        }
    };

    match String::from_utf8(bytes) {
        Ok(text) => ConfigReadOutcome::Ok(text),
        Err(_) => ConfigReadOutcome::Refused(RefusalReason::NotUtf8),
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
    let file = open_read_only_no_follow_nonblocking(path)
        .map_err(|error| classify_open_error(path, error))?;
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

fn classify_open_error(path: &Path, error: std::io::Error) -> BoundedReadError {
    // O_NOFOLLOW reports a platform-specific error kind. Inspecting the path
    // here is only for a value-free refusal label; an open error always stays
    // a refusal regardless of any subsequent path race.
    if std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        BoundedReadError::Symlink
    } else {
        classify_io_error(error)
    }
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
    const FILE_SHARE_READ: u32 = 0x1;
    const FILE_SHARE_DELETE: u32 = 0x4;
    options
        .share_mode(FILE_SHARE_READ | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
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
    fn regular_config_replacement_after_open_reads_stable_original_handle() {
        let (_dir, root) = root_with(Some(b"[history]\npersistence = \"none\"\n"));
        let config = root.config_path();
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
