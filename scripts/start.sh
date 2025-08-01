#!/usr/bin/env bash
set -euo pipefail

echo "[+] Building binaries"
make build

echo "[+] Bringing up docker-compose"
docker compose up --build
