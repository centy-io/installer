use std::env::consts::{ARCH, OS};

pub struct Platform {
    pub target: &'static str,
    pub archive_ext: &'static str,
}

pub fn detect() -> Result<Platform, String> {
    let target = match (OS, ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        _ => return Err(format!("unsupported platform: {OS}-{ARCH}")),
    };

    let archive_ext = match OS {
        "windows" => ".zip",
        _ => ".tar.gz",
    };

    Ok(Platform { target, archive_ext })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn detect_current_platform() {
        let platform = detect().expect("current platform should be supported");
        assert!(!platform.target.is_empty());
        assert!(!platform.archive_ext.is_empty());
    }

    #[test]
    fn archive_ext_matches_os() {
        let platform = detect().unwrap();
        if cfg!(target_os = "windows") {
            assert_eq!(platform.archive_ext, ".zip");
        } else {
            assert_eq!(platform.archive_ext, ".tar.gz");
        }
    }
}
