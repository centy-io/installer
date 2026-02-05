use reqwest::blocking::Client;

use crate::platform::Platform;

const REPO: &str = "centy-io/centy-daemon";

pub struct ReleaseInfo {
    pub tag: String,
    pub asset_url: String,
    pub checksums_url: String,
    pub asset_name: String,
}

/// Resolve the version tag to use. If `version` is None, fetch the latest release
/// (including pre-releases) from the GitHub API.
pub fn resolve_version(client: &Client, version: Option<&str>) -> Result<String, String> {
    if let Some(v) = version {
        let tag = if v.starts_with('v') {
            v.to_string()
        } else {
            format!("v{v}")
        };
        return Ok(tag);
    }

    // Fetch all releases and pick the first one (most recent, includes pre-releases)
    let url = format!("https://api.github.com/repos/{REPO}/releases");
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
    let version_num = tag.strip_prefix('v').unwrap_or(tag);
    let asset_name = format!(
        "centy-daemon-{version_num}-{}{}",
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
    fn parse_checksum_not_found() {
        let checksums = "abc123  other-file.tar.gz\n";
        let result = parse_checksum(checksums, "missing.tar.gz");
        assert!(result.is_err());
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
            "centy-daemon-0.2.0-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            info.asset_url,
            "https://github.com/centy-io/centy-daemon/releases/download/v0.2.0/centy-daemon-0.2.0-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            info.checksums_url,
            "https://github.com/centy-io/centy-daemon/releases/download/v0.2.0/checksums-sha256.txt"
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
}
