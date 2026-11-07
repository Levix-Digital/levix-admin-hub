#!/usr/bin/env bash
# Starts a fresh Hub with the embedded console, runs the browser tests against it, stops it.
# Usage: console-tests/run.sh [path to levix-hub binary]
set -euo pipefail
binary="$(realpath "${1:-$(dirname "$0")/../target/debug/levix-hub}")"
cd "$(dirname "$0")"
data="$(mktemp -d)"
empty="$(mktemp -d)"
port="${HUB_PORT:-18443}"
empty_port=$((port + 1))
trap 'kill $(jobs -p) 2>/dev/null || true; rm -rf "$data" "$empty"' EXIT

init="$("$binary" --data-dir "$data" init --owner-email owner@example.com \
  --public-url "https://localhost:$port" --hostnames localhost)"
export HUB_URL="https://localhost:$port"
export HUB_SETUP_LINK="$(grep -o "https://[^ ]*#/link/[^ ]*" <<<"$init")"
export HUB_EMPTY_URL="https://localhost:$empty_port"

"$binary" --data-dir "$data" serve --listen "127.0.0.1:$port" --log warn &
"$binary" --data-dir "$empty" serve --listen "127.0.0.1:$empty_port" --log warn &
for _ in $(seq 50); do
  curl -sfk "$HUB_URL/readyz" >/dev/null && curl -sfk "$HUB_EMPTY_URL/healthz" >/dev/null && break
  sleep 0.2
done
npx playwright test --trace off
