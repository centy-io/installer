//! End-to-end tests for the centy-daemon installation pipeline.
//!
//! These tests download real releases from GitHub and verify the full
//! install flow: version resolution, download, checksum, extract, install.
//!
//! Run with: `cargo test --test e2e_install -- --test-threads=1`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

fn binary_path() -> PathBuf {
    let name = if cfg!(target_os = "windows") {
        "centy-daemon.exe"
    } else {
        "centy-daemon"
    };
    dirs::home_dir()
        .expect("home directory must exist")
        .join(".centy")
        .join("bin")
        .join(name)
}

fn cleanup() {
    let path = binary_path();
    if path.exists() {
        fs::remove_file(&path).expect("failed to remove existing binary");
    }
}

#[test]
fn install_pinned_version() {
    cleanup();

    let path =
        centy_installer::install(Some("v0.1.6"), false, false).expect("install v0.1.6 should succeed");

    assert!(path.exists(), "binary should exist at {}", path.display());

    let metadata = fs::metadata(&path).expect("should read binary metadata");
    assert!(metadata.len() > 0, "binary should be non-empty");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        assert_eq!(
            mode & 0o111,
            0o111,
            "binary should have executable bits set"
        );
    }
}

#[test]
fn install_latest_version() {
    cleanup();

    let path = centy_installer::install(None, false, false).expect("install latest should succeed");

    assert!(path.exists(), "binary should exist at {}", path.display());
}

#[test]
fn install_version_without_v_prefix() {
    cleanup();

    let path = centy_installer::install(Some("0.1.6"), false, false)
        .expect("install without v prefix should succeed");

    assert!(path.exists(), "binary should exist at {}", path.display());
}

#[test]
fn install_nonexistent_version_fails() {
    cleanup();

    let result = centy_installer::install(Some("v99.99.99"), false, false);

    assert!(
        result.is_err(),
        "install of nonexistent version should fail"
    );
}
