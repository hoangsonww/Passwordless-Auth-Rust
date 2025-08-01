#!/usr/bin/env bash
set -euo pipefail

echo "[+] Running rustfmt (format in place)"
cargo fmt

echo "[+] Running clippy (treat warnings as errors)"
cargo clippy --all-targets --all-features -- -D warnings

echo "[+] Format & lint completed successfully"
