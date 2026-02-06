# installer

Centy installer — installs the `centy-daemon` binary. Available as a shell script, npm package, cargo crate, or go module. Each ecosystem shell wraps a core Rust library that handles the install logic.

## Shell Script Install (recommended)

```bash
curl -fsSL https://github.com/centy-io/installer/releases/latest/download/install.sh | sh
```

Install a specific version:

```bash
curl -fsSL https://github.com/centy-io/installer/releases/latest/download/install.sh | sh -s v0.1.0
```

The script detects your OS and architecture, downloads the correct `centy-installer` binary, and runs it to install `centy-daemon` to `~/.centy/bin/`.

### Supported Platforms

| OS      | Architecture    |
|---------|-----------------|
| macOS   | Intel (`x86_64`), Apple Silicon (`aarch64`) |
| Linux   | `x86_64`, `aarch64` |

## Other Install Methods

```bash
# Via npm
npx centy-installer

# Via cargo
cargo install centy-installer && centy-installer
```
