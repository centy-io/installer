use std::fs;
use std::path::PathBuf;

/// Install the binary bytes to `~/.centy/bin/centy-daemon` and return the path.
pub fn install_binary(binary_bytes: &[u8]) -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("could not determine home directory")?;
    let bin_dir = home.join(".centy").join("bin");

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
