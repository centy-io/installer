mod download;
mod extract;
mod github;
mod install;
mod platform;

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum InstallerError {
    #[error("platform detection failed: {0}")]
    Platform(String),

    #[error("version resolution failed: {0}")]
    VersionResolution(String),

    #[error("download failed: {0}")]
    Download(String),

    #[error("extraction failed: {0}")]
    Extraction(String),

    #[error("installation failed: {0}")]
    Installation(String),
}

/// Download and install the `centy-daemon` binary.
///
/// If `version` is `None`, the latest release (including pre-releases) is used.
/// Returns the path to the installed binary (`~/.centy/bin/centy-daemon`).
pub fn install(version: Option<&str>) -> Result<PathBuf, InstallerError> {
    let platform = platform::detect().map_err(InstallerError::Platform)?;

    let client = reqwest::blocking::Client::new();

    let tag = github::resolve_version(&client, version)
        .map_err(InstallerError::VersionResolution)?;

    let info = github::release_info(&tag, &platform);

    let asset = download::download_and_verify(&client, &info)
        .map_err(InstallerError::Download)?;

    let binary_bytes = match platform.archive_ext {
        ".tar.gz" => extract::extract_tar_gz(&asset.bytes),
        ".zip" => extract::extract_zip(&asset.bytes),
        ext => Err(format!("unsupported archive format: {ext}")),
    }
    .map_err(InstallerError::Extraction)?;

    let path = install::install_binary(&binary_bytes)
        .map_err(InstallerError::Installation)?;

    Ok(path)
}
