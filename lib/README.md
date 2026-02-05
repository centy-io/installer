# centy-installer

Core Rust library containing the shared installation logic for the `centy-daemon` binary. Each ecosystem shell (npm, cargo, go) wraps this library.

## Usage

```rust
use centy_installer::{install, InstallerError};

// Install the latest version
let path = install(None)?;

// Install a specific version
let path = install(Some("0.1.0"))?;

println!("Installed to {}", path.display());
// => ~/.centy/bin/centy-daemon
```

## How it works

The `install` function runs through five steps:

1. **Platform detection** — identifies OS and architecture (macOS, Linux, Windows on x86_64/aarch64)
2. **Version resolution** — resolves the requested version tag, or fetches the latest release from the GitHub API
3. **Download & verify** — downloads the release archive and its SHA-256 checksums file, then verifies integrity
4. **Extraction** — extracts the `centy-daemon` binary from the archive (`.tar.gz` on Unix, `.zip` on Windows)
5. **Installation** — writes the binary to `~/.centy/bin/centy-daemon` with executable permissions

## API

### `install(version: Option<&str>) -> Result<PathBuf, InstallerError>`

Downloads and installs the `centy-daemon` binary. Pass `None` to install the latest release (including pre-releases), or `Some("x.y.z")` to pin a version. Returns the path to the installed binary.

### `InstallerError`

```rust
pub enum InstallerError {
    Platform(String),
    VersionResolution(String),
    Download(String),
    Extraction(String),
    Installation(String),
}
```

## Platform support

| OS      | Architecture       | Archive format |
|---------|--------------------|----------------|
| macOS   | aarch64, x86_64    | `.tar.gz`      |
| Linux   | aarch64, x86_64    | `.tar.gz`      |
| Windows | aarch64, x86_64    | `.zip`         |

## License

MIT
