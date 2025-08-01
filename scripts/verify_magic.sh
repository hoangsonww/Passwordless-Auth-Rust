#!/usr/bin/env bash
set -euo pipefail

TOKEN=${1:?token required}
echo "[+] Verifying magic token"
curl -s "http://localhost:3000/verify/magic?token=${TOKEN}" | jq
