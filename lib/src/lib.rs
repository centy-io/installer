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

pub(crate) fn extract_binary(
    archive_bytes: &[u8],
    archive_ext: &str,
) -> Result<Vec<u8>, InstallerError> {
    match archive_ext {
        ".tar.gz" => extract::extract_tar_gz(archive_bytes),
        ".zip" => extract::extract_zip(archive_bytes),
        ext => Err(format!("unsupported archive format: {ext}")),
    }
    .map_err(InstallerError::Extraction)
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

    let binary_bytes = extract_binary(&asset.bytes, platform.archive_ext)?;

    let path = install::install_binary(&binary_bytes)
        .map_err(InstallerError::Installation)?;

    Ok(path)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn error_display_platform() {
        let err = InstallerError::Platform("unsupported os".to_string());
        assert_eq!(err.to_string(), "platform detection failed: unsupported os");
    }

    #[test]
    fn error_display_version_resolution() {
        let err = InstallerError::VersionResolution("no releases".to_string());
        assert_eq!(
            err.to_string(),
            "version resolution failed: no releases"
        );
    }

    #[test]
    fn error_display_download() {
        let err = InstallerError::Download("connection refused".to_string());
        assert_eq!(err.to_string(), "download failed: connection refused");
    }

    #[test]
    fn error_display_extraction() {
        let err = InstallerError::Extraction("corrupt archive".to_string());
        assert_eq!(err.to_string(), "extraction failed: corrupt archive");
    }

    #[test]
    fn error_display_installation() {
        let err = InstallerError::Installation("permission denied".to_string());
        assert_eq!(
            err.to_string(),
            "installation failed: permission denied"
        );
    }

    #[test]
    fn error_is_debug() {
        let err = InstallerError::Platform("test".to_string());
        let debug = format!("{err:?}");
        assert!(debug.contains("Platform"));
    }

    #[test]
    fn extract_binary_tar_gz() {
        use std::io::Write;

        let mut tar_builder = tar::Builder::new(Vec::new());
        let content = b"binary-data";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar_builder
            .append_data(&mut header, "centy-daemon", &content[..])
            .unwrap();
        let tar_bytes = tar_builder.into_inner().unwrap();

        let mut encoder =
            flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        let gz_bytes = encoder.finish().unwrap();

        let result = extract_binary(&gz_bytes, ".tar.gz").unwrap();
        assert_eq!(result, b"binary-data");
    }

    #[test]
    fn extract_binary_zip() {
        use std::io::{Cursor, Write};

        let buf = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("centy-daemon", options).unwrap();
        zip.write_all(b"zip-binary").unwrap();
        let zip_bytes = zip.finish().unwrap().into_inner();

        let result = extract_binary(&zip_bytes, ".zip").unwrap();
        assert_eq!(result, b"zip-binary");
    }

    #[test]
    fn extract_binary_unsupported_format() {
        let result = extract_binary(b"data", ".rar");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("unsupported archive format: .rar"));
    }

    #[test]
    fn extract_binary_tar_gz_missing_binary() {
        use std::io::Write;

        let mut tar_builder = tar::Builder::new(Vec::new());
        let content = b"other";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar_builder
            .append_data(&mut header, "other-file", &content[..])
            .unwrap();
        let tar_bytes = tar_builder.into_inner().unwrap();

        let mut encoder =
            flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        let gz_bytes = encoder.finish().unwrap();

        let result = extract_binary(&gz_bytes, ".tar.gz");
        assert!(result.is_err());
        match result.unwrap_err() {
            InstallerError::Extraction(msg) => {
                assert!(msg.contains("not found"));
            }
            other => panic!("expected Extraction error, got: {other:?}"),
        }
    }
}
