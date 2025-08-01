#!/usr/bin/env bash
set -euo pipefail

EMAIL=${1:-test@example.com}
echo "[+] Requesting magic link for $EMAIL"
curl -s -X POST http://localhost:3000/request/magic \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"${EMAIL}\"}" | jq
