#!/usr/bin/env bash
set -euo pipefail

export RUST_LOG=info
echo "[+] Running unit/integration tests"
cargo test
