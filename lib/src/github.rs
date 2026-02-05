use reqwest::blocking::Client;

use crate::platform::Platform;

const REPO: &str = "centy-io/centy-daemon";

pub struct ReleaseInfo {
    /// Retained for consumers that need the resolved tag (e.g. for display/logging).
    #[allow(dead_code)]
    pub tag: String,
    pub asset_url: String,
    pub checksums_url: String,
    pub asset_name: String,
}

/// Resolve the version tag to use. If `version` is None, fetch the latest release
/// (including pre-releases) from the GitHub API.
pub fn resolve_version(client: &Client, version: Option<&str>) -> Result<String, String> {
    resolve_version_from(client, version, "https://api.github.com")
}

pub fn resolve_version_from(
    client: &Client,
    version: Option<&str>,
    api_base: &str,
) -> Result<String, String> {
    if let Some(v) = version {
        let tag = if v.starts_with('v') {
            v.to_string()
        } else {
            format!("v{v}")
        };
        return Ok(tag);
    }

    // Fetch all releases and pick the first one (most recent, includes pre-releases)
    let url = format!("{api_base}/repos/{REPO}/releases");
    let resp = client
        .get(&url)
        .header("User-Agent", "centy-installer")
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| format!("failed to fetch releases: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {}", resp.status()));
    }

    let text = resp
        .text()
        .map_err(|e| format!("failed to read response body: {e}"))?;
    let body: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse releases JSON: {e}"))?;

    let tag = body
        .as_array()
        .and_then(|releases| releases.first())
        .and_then(|r| r["tag_name"].as_str())
        .ok_or("no releases found")?
        .to_string();

    Ok(tag)
}

/// Build release info (download URLs) for the given version tag and platform.
pub fn release_info(tag: &str, platform: &Platform) -> ReleaseInfo {
    let asset_name = format!(
        "centy-daemon-{tag}-{}{}",
        platform.target, platform.archive_ext
    );
    let base = format!("https://github.com/{REPO}/releases/download/{tag}");

    ReleaseInfo {
        tag: tag.to_string(),
        asset_url: format!("{base}/{asset_name}"),
        checksums_url: format!("{base}/checksums-sha256.txt"),
        asset_name,
    }
}

/// Parse checksums-sha256.txt and return the expected hash for the given asset name.
pub fn parse_checksum(checksums_text: &str, asset_name: &str) -> Result<String, String> {
    for line in checksums_text.lines() {
        // Format: "<hash>  <filename>" or "<hash> <filename>"
        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() == 2 {
            let filename = parts[1].trim();
            if filename == asset_name {
                return Ok(parts[0].to_string());
            }
        }
    }
    Err(format!(
        "checksum not found for {asset_name} in checksums file"
    ))
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

    #[test]
    fn parse_checksum_found() {
        let checksums = "\
abc123  centy-daemon-0.1.0-aarch64-apple-darwin.tar.gz
def456  centy-daemon-0.1.0-x86_64-unknown-linux-gnu.tar.gz
";
        let hash =
            parse_checksum(checksums, "centy-daemon-0.1.0-aarch64-apple-darwin.tar.gz").unwrap();
        assert_eq!(hash, "abc123");
    }

    #[test]
    fn parse_checksum_second_entry() {
        let checksums = "\
abc123  centy-daemon-0.1.0-aarch64-apple-darwin.tar.gz
def456  centy-daemon-0.1.0-x86_64-unknown-linux-gnu.tar.gz
";
        let hash = parse_checksum(
            checksums,
            "centy-daemon-0.1.0-x86_64-unknown-linux-gnu.tar.gz",
        )
        .unwrap();
        assert_eq!(hash, "def456");
    }

    #[test]
    fn parse_checksum_single_space_separator() {
        let checksums = "abc123 my-asset.tar.gz\n";
        let hash = parse_checksum(checksums, "my-asset.tar.gz").unwrap();
        assert_eq!(hash, "abc123");
    }

    #[test]
    fn parse_checksum_not_found() {
        let checksums = "abc123  other-file.tar.gz\n";
        let result = parse_checksum(checksums, "missing.tar.gz");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("checksum not found for missing.tar.gz"));
    }

    #[test]
    fn parse_checksum_empty_input() {
        let result = parse_checksum("", "anything.tar.gz");
        assert!(result.is_err());
    }

    #[test]
    fn parse_checksum_blank_lines() {
        let checksums = "\n\nabc123  target.tar.gz\n\n";
        let hash = parse_checksum(checksums, "target.tar.gz").unwrap();
        assert_eq!(hash, "abc123");
    }

    #[test]
    fn release_info_builds_urls() {
        let platform = Platform {
            target: "aarch64-apple-darwin",
            archive_ext: ".tar.gz",
        };
        let info = release_info("v0.2.0", &platform);
        assert_eq!(
            info.asset_name,
            "centy-daemon-v0.2.0-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            info.asset_url,
            "https://github.com/centy-io/centy-daemon/releases/download/v0.2.0/centy-daemon-v0.2.0-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            info.checksums_url,
            "https://github.com/centy-io/centy-daemon/releases/download/v0.2.0/checksums-sha256.txt"
        );
        assert_eq!(info.tag, "v0.2.0");
    }

    #[test]
    fn release_info_tag_without_v_prefix() {
        let platform = Platform {
            target: "x86_64-unknown-linux-gnu",
            archive_ext: ".tar.gz",
        };
        let info = release_info("1.0.0", &platform);
        assert_eq!(
            info.asset_name,
            "centy-daemon-1.0.0-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            info.asset_url,
            "https://github.com/centy-io/centy-daemon/releases/download/1.0.0/centy-daemon-1.0.0-x86_64-unknown-linux-gnu.tar.gz"
        );
    }

    #[test]
    fn release_info_tag_with_v_prefix_preserves_v_in_asset() {
        let platform = Platform {
            target: "x86_64-unknown-linux-gnu",
            archive_ext: ".tar.gz",
        };
        let info = release_info("v1.0.0", &platform);
        assert_eq!(
            info.asset_name,
            "centy-daemon-v1.0.0-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            info.asset_url,
            "https://github.com/centy-io/centy-daemon/releases/download/v1.0.0/centy-daemon-v1.0.0-x86_64-unknown-linux-gnu.tar.gz"
        );
    }

    #[test]
    fn release_info_windows_zip() {
        let platform = Platform {
            target: "x86_64-pc-windows-msvc",
            archive_ext: ".zip",
        };
        let info = release_info("v0.3.0", &platform);
        assert_eq!(
            info.asset_name,
            "centy-daemon-v0.3.0-x86_64-pc-windows-msvc.zip"
        );
    }

    #[test]
    fn resolve_version_with_v_prefix() {
        let client = Client::new();
        let tag = resolve_version(&client, Some("v1.0.0")).unwrap();
        assert_eq!(tag, "v1.0.0");
    }

    #[test]
    fn resolve_version_without_v_prefix() {
        let client = Client::new();
        let tag = resolve_version(&client, Some("1.0.0")).unwrap();
        assert_eq!(tag, "v1.0.0");
    }

    #[test]
    fn resolve_version_none_fetches_latest() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/repos/centy-io/centy-daemon/releases")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"tag_name": "v0.5.0"}, {"tag_name": "v0.4.0"}]"#)
            .create();

        let client = Client::new();
        let tag = resolve_version_from(&client, None, &server.url()).unwrap();
        assert_eq!(tag, "v0.5.0");
        mock.assert();
    }

    #[test]
    fn resolve_version_none_api_error() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/repos/centy-io/centy-daemon/releases")
            .with_status(403)
            .create();

        let client = Client::new();
        let result = resolve_version_from(&client, None, &server.url());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("GitHub API returned 403"));
        mock.assert();
    }

    #[test]
    fn resolve_version_none_invalid_json() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/repos/centy-io/centy-daemon/releases")
            .with_status(200)
            .with_body("not-json")
            .create();

        let client = Client::new();
        let result = resolve_version_from(&client, None, &server.url());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("failed to parse releases JSON"));
        mock.assert();
    }

    #[test]
    fn resolve_version_none_empty_releases() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/repos/centy-io/centy-daemon/releases")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create();

        let client = Client::new();
        let result = resolve_version_from(&client, None, &server.url());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no releases found"));
        mock.assert();
    }

    #[test]
    fn resolve_version_none_missing_tag_name() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/repos/centy-io/centy-daemon/releases")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"name": "Release 1"}]"#)
            .create();

        let client = Client::new();
        let result = resolve_version_from(&client, None, &server.url());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no releases found"));
        mock.assert();
    }

    #[test]
    fn resolve_version_from_with_version_ignores_api_base() {
        // When a version is provided, the API base is never used
        let client = Client::new();
        let tag =
            resolve_version_from(&client, Some("2.0.0"), "http://invalid-url.example.com")
                .unwrap();
        assert_eq!(tag, "v2.0.0");
    }
}
