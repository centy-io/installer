use std::io::{Cursor, Read};

/// Extract the `centy-daemon` binary from a `.tar.gz` archive.
pub fn extract_tar_gz(archive_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let decoder = flate2::read::GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = tar::Archive::new(decoder);

    for entry in archive
        .entries()
        .map_err(|e| format!("failed to read tar entries: {e}"))?
    {
        let mut entry = entry.map_err(|e| format!("failed to read tar entry: {e}"))?;
        let path = entry
            .path()
            .map_err(|e| format!("failed to read entry path: {e}"))?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name == "centy-daemon" {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("failed to read binary from archive: {e}"))?;
            return Ok(buf);
        }
    }

    Err("centy-daemon binary not found in tar.gz archive".to_string())
}

/// Extract the `centy-daemon` binary from a `.zip` archive.
pub fn extract_zip(archive_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let reader = Cursor::new(archive_bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("failed to open zip archive: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("failed to read zip entry: {e}"))?;

        let file_name = file
            .enclosed_name()
            .and_then(|p| p.file_name().map(|n| n.to_os_string()))
            .and_then(|n| n.into_string().ok())
            .unwrap_or_default();

        if file_name == "centy-daemon" || file_name == "centy-daemon.exe" {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| format!("failed to read binary from zip: {e}"))?;
            return Ok(buf);
        }
    }

    Err("centy-daemon binary not found in zip archive".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extract_tar_gz_finds_binary() {
        // Create a tar.gz in memory with a fake "centy-daemon" file
        let mut tar_builder = tar::Builder::new(Vec::new());
        let content = b"fake-binary-content";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar_builder
            .append_data(&mut header, "centy-daemon", &content[..])
            .unwrap();
        let tar_bytes = tar_builder.into_inner().unwrap();

        // Compress with gzip
        let mut encoder =
            flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&tar_bytes).unwrap();
        let gz_bytes = encoder.finish().unwrap();

        let result = extract_tar_gz(&gz_bytes).unwrap();
        assert_eq!(result, b"fake-binary-content");
    }

    #[test]
    fn extract_tar_gz_missing_binary() {
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

        let result = extract_tar_gz(&gz_bytes);
        assert!(result.is_err());
    }
}
