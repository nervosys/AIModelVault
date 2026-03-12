//! Cross-platform file permission utilities.
//!
//! On Unix, sets POSIX mode bits (0o600 for files, 0o700 for directories).
//! On Windows, restricts NTFS ACLs to the current user via `icacls`.

use std::path::Path;

/// Restrict a file to owner-read/write only (Unix 0o600 equivalent).
pub fn restrict_file(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
    }
    #[cfg(windows)]
    {
        restrict_acl(path)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Ok(())
    }
}

/// Restrict a directory to owner-only access (Unix 0o700 equivalent).
pub fn restrict_dir(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
    }
    #[cfg(windows)]
    {
        restrict_acl(path)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Ok(())
    }
}

/// Set restrictive mode on [`std::fs::OpenOptions`] (Unix only, no-op on other platforms).
///
/// Call this before `.open()` to atomically create files with `0o600` permissions
/// on Unix. On Windows, call [`restrict_file`] after the file is created instead.
#[cfg(unix)]
pub fn set_create_mode(opts: &mut std::fs::OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    opts.mode(0o600);
}

/// No-op on non-Unix platforms — use [`restrict_file`] after file creation.
#[cfg(not(unix))]
pub fn set_create_mode(_opts: &mut std::fs::OpenOptions) {}

/// Restrict NTFS ACLs to the current user via `icacls`.
///
/// Removes inherited ACEs and grants Full Control only to `%USERNAME%`.
#[cfg(windows)]
fn restrict_acl(path: &Path) -> std::io::Result<()> {
    let username = std::env::var("USERNAME").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "USERNAME environment variable not set",
        )
    })?;

    let status = std::process::Command::new("icacls")
        .arg(path.as_os_str())
        .arg("/inheritance:r")
        .arg("/grant:r")
        .arg(format!("{username}:F"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Failed to restrict file permissions via icacls",
        ));
    }
    Ok(())
}
