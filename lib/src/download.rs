use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

use crate::github::ReleaseInfo;

pub struct DownloadedAsset {
    pub bytes: Vec<u8>,
}

/// Download the asset archive and verify its SHA256 checksum.
pub fn download_and_verify(client: &Client, info: &ReleaseInfo) -> Result<DownloadedAsset, String> {
    // Download checksums file
    let checksums_text = client
        .get(&info.checksums_url)
        .header("User-Agent", "centy-installer")
        .send()
        .and_then(|r| r.text())
        .map_err(|e| format!("failed to download checksums: {e}"))?;

    let expected_hash = crate::github::parse_checksum(&checksums_text, &info.asset_name)?;

    // Download asset archive
    let asset_bytes = client
        .get(&info.asset_url)
        .header("User-Agent", "centy-installer")
        .send()
        .and_then(|r| r.bytes())
        .map_err(|e| format!("failed to download asset: {e}"))?
        .to_vec();

    // Verify checksum
    let mut hasher = Sha256::new();
    hasher.update(&asset_bytes);
    let actual_hash = hex::encode(hasher.finalize());

    if actual_hash != expected_hash {
        return Err(format!(
            "checksum mismatch: expected {expected_hash}, got {actual_hash}"
        ));
    }

    Ok(DownloadedAsset { bytes: asset_bytes })
}
