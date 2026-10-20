#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
ASSETS="${1:-target/ownership-rd}"
DIST="${2:-target/ownership-rd-web}"
NOIR_PREPARED="${3:-target/groth16-tribute}"
DEMO_PREPARED="${4:-target/ownership-rd-demo}"
if [[ ! -f "$ASSETS/manifest.json" ]]; then
  echo "Missing $ASSETS/manifest.json; run ownership-rd prepare first." >&2
  exit 1
fi
if [[ ! -f "$NOIR_PREPARED/preparation.json" ]]; then
  echo "Missing $NOIR_PREPARED/preparation.json; run groth16-tribute prepare or pass its prepared directory as argument 3." >&2
  exit 1
fi
if [[ ! -f "$DEMO_PREPARED/manifest.json" ]]; then
  echo "Missing $DEMO_PREPARED/manifest.json; run ownership-rd prepare-demo --scheme poseidon2 --out \"$DEMO_PREPARED\", or pass its prepared directory as argument 4 (default: target/ownership-rd-demo)." >&2
  exit 1
fi
mkdir -p "$DIST"
DIST="$(cd "$DIST" && pwd)"
case "$DIST/" in "$ROOT/target/"*) ;; *) echo "Generated demo must stay under target/." >&2; exit 1;; esac
rustup target add wasm32-unknown-unknown
wasm-pack build experiments/ownership-rd --release --target web --out-dir "$DIST/pkg" -- --features wasm
cp experiments/ownership-rd/web/index.html experiments/ownership-rd/web/styles.css experiments/ownership-rd/web/app.js experiments/ownership-rd/web/worker.js "$DIST/"
mkdir -p "$DIST/assets"
cp "$ASSETS/manifest.json" "$DIST/assets/manifest.json"
# Replace only this generated asset directory: never overwrite a hardlinked
# prepared key when rebuilding the demo from a different parameter set.
rm -rf "$DIST/assets/keys"
# Remove the obsolete unqualified Demo key paths before staging scheme-specific keys.
rm -rf "$DIST/assets/demo_tribute/groth16"
mkdir -p "$DIST/assets/keys"
cp -R -l "$ASSETS/keys/." "$DIST/assets/keys/" 2>/dev/null || cp -R "$ASSETS/keys/." "$DIST/assets/keys/"
(cd experiments/ownership-rd && npm ci --ignore-scripts --no-audit --no-fund)
node experiments/ownership-rd/build-noir.mjs "$NOIR_PREPARED" "$DIST" "$DEMO_PREPARED"
printf '\nDemo built at %s\nServe it with:\n  python3 -m http.server 8080 --bind 0.0.0.0 --directory "%s"\n' "$DIST" "$DIST"
printf 'Open http://localhost:8080 locally or http://YOUR-LAN-IP:8080 on a phone in the same trusted network. Public fixtures only; use HTTPS on untrusted networks.\n'
