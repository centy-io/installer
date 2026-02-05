#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

docker build -f "$REPO_ROOT/e2e/Dockerfile.linux" -t centy-e2e-linux "$REPO_ROOT"
docker run --rm centy-e2e-linux
