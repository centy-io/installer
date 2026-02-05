use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

use crate::github::ReleaseInfo;

#[derive(Debug)]
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
        .and_then(reqwest::blocking::Response::text)
        .map_err(|e| format!("failed to download checksums: {e}"))?;

    let expected_hash = crate::github::parse_checksum(&checksums_text, &info.asset_name)?;

    // Download asset archive
    let asset_bytes = client
        .get(&info.asset_url)
        .header("User-Agent", "centy-installer")
        .send()
        .and_then(reqwest::blocking::Response::bytes)
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

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::significant_drop_tightening
)]
mod tests {
    use super::*;

    fn make_info(server_url: &str) -> ReleaseInfo {
        ReleaseInfo {
            tag: "v1.0.0".to_string(),
            asset_url: format!("{server_url}/test-asset.tar.gz"),
            checksums_url: format!("{server_url}/checksums-sha256.txt"),
            asset_name: "test-asset.tar.gz".to_string(),
        }
    }

    #[test]
    fn download_and_verify_success() {
        let mut server = mockito::Server::new();

        let asset_bytes = b"fake-binary-data";
        let mut hasher = Sha256::new();
        hasher.update(asset_bytes);
        let expected_hash = hex::encode(hasher.finalize());

        let checksums_body = format!("{expected_hash}  test-asset.tar.gz\n");

        let checksums_mock = server
            .mock("GET", "/checksums-sha256.txt")
            .with_status(200)
            .with_body(&checksums_body)
            .create();

        let asset_mock = server
            .mock("GET", "/test-asset.tar.gz")
            .with_status(200)
            .with_body(asset_bytes)
            .create();

        let client = Client::new();
        let info = make_info(&server.url());
        let result = download_and_verify(&client, &info).unwrap();
        assert_eq!(result.bytes, asset_bytes);

        checksums_mock.assert();
        asset_mock.assert();
    }

    #[test]
    fn download_and_verify_checksum_mismatch() {
        let mut server = mockito::Server::new();

        let checksums_body = "deadbeef00000000000000000000000000000000000000000000000000000000  test-asset.tar.gz\n";

        server
            .mock("GET", "/checksums-sha256.txt")
            .with_status(200)
            .with_body(checksums_body)
            .create();

        server
            .mock("GET", "/test-asset.tar.gz")
            .with_status(200)
            .with_body("some-data")
            .create();

        let client = Client::new();
        let info = make_info(&server.url());
        let result = download_and_verify(&client, &info);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("checksum mismatch"));
    }

    #[test]
    fn download_and_verify_asset_not_in_checksums() {
        let mut server = mockito::Server::new();

        // Checksums file doesn't contain our asset name
        let checksums_body = "abc123  other-asset.tar.gz\n";

        server
            .mock("GET", "/checksums-sha256.txt")
            .with_status(200)
            .with_body(checksums_body)
            .create();

        let client = Client::new();
        let info = make_info(&server.url());
        let result = download_and_verify(&client, &info);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("checksum not found"));
    }

    #[test]
    fn download_and_verify_checksums_connection_error() {
        let info = ReleaseInfo {
            tag: "v1.0.0".to_string(),
            asset_url: "http://127.0.0.1:1/asset.tar.gz".to_string(),
            checksums_url: "http://127.0.0.1:1/checksums-sha256.txt".to_string(),
            asset_name: "asset.tar.gz".to_string(),
        };

        let client = Client::new();
        let result = download_and_verify(&client, &info);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("failed to download checksums"));
    }

    #[test]
    fn download_and_verify_asset_connection_error() {
        let mut server = mockito::Server::new();

        let asset_bytes = b"data";
        let mut hasher = Sha256::new();
        hasher.update(asset_bytes);
        let expected_hash = hex::encode(hasher.finalize());

        let checksums_body = format!("{expected_hash}  test-asset.tar.gz\n");

        server
            .mock("GET", "/checksums-sha256.txt")
            .with_status(200)
            .with_body(&checksums_body)
            .create();

        // Asset URL points to a closed port
        let info = ReleaseInfo {
            tag: "v1.0.0".to_string(),
            asset_url: "http://127.0.0.1:1/test-asset.tar.gz".to_string(),
            checksums_url: format!("{}/checksums-sha256.txt", server.url()),
            asset_name: "test-asset.tar.gz".to_string(),
        };

        let client = Client::new();
        let result = download_and_verify(&client, &info);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("failed to download asset"));
    }
}
