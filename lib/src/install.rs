use std::fs;
use std::path::{Path, PathBuf};

/// Install the binary bytes to `~/.centy/bin/centy-daemon` and return the path.
pub fn install_binary(binary_bytes: &[u8]) -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("could not determine home directory")?;
    install_binary_to(binary_bytes, &home)
}

pub fn install_binary_to(binary_bytes: &[u8], home_dir: &Path) -> Result<PathBuf, String> {
    let bin_dir = home_dir.join(".centy").join("bin");

    fs::create_dir_all(&bin_dir)
        .map_err(|e| format!("failed to create {}: {e}", bin_dir.display()))?;

    let binary_name = if cfg!(target_os = "windows") {
        "centy-daemon.exe"
    } else {
        "centy-daemon"
    };
    let binary_path = bin_dir.join(binary_name);

    fs::write(&binary_path, binary_bytes)
        .map_err(|e| format!("failed to write binary to {}: {e}", binary_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&binary_path, perms)
            .map_err(|e| format!("failed to set permissions: {e}"))?;
    }

    Ok(binary_path)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn install_binary_to_success() {
        let tmp = tempfile::tempdir().unwrap();
        let binary_bytes = b"test-binary-content";

        let path = install_binary_to(binary_bytes, tmp.path()).unwrap();

        assert!(path.exists());
        assert_eq!(fs::read(&path).unwrap(), binary_bytes);

        if cfg!(target_os = "windows") {
            assert!(path.ends_with("centy-daemon.exe"));
        } else {
            assert!(path.ends_with("centy-daemon"));
        }
    }

    #[test]
    fn install_binary_to_creates_directories() {
        let tmp = tempfile::tempdir().unwrap();

        // The .centy/bin directory should not exist yet
        let bin_dir = tmp.path().join(".centy").join("bin");
        assert!(!bin_dir.exists());

        install_binary_to(b"data", tmp.path()).unwrap();

        assert!(bin_dir.exists());
    }

    #[test]
    fn install_binary_to_overwrites_existing() {
        let tmp = tempfile::tempdir().unwrap();

        let path = install_binary_to(b"first-version", tmp.path()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"first-version");

        let path = install_binary_to(b"second-version", tmp.path()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second-version");
    }

    #[cfg(unix)]
    #[test]
    fn install_binary_to_sets_executable_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let path = install_binary_to(b"binary", tmp.path()).unwrap();

        let metadata = fs::metadata(&path).unwrap();
        let mode = metadata.permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
    }

    #[test]
    fn install_binary_to_returns_correct_path() {
        let tmp = tempfile::tempdir().unwrap();
        let path = install_binary_to(b"data", tmp.path()).unwrap();

        let expected = tmp.path().join(".centy").join("bin").join(if cfg!(target_os = "windows") {
            "centy-daemon.exe"
        } else {
            "centy-daemon"
        });

        assert_eq!(path, expected);
    }

    #[test]
    fn install_binary_to_invalid_path() {
        let result = install_binary_to(b"data", Path::new("/nonexistent/invalid/path"));
        assert!(result.is_err());
    }
}
