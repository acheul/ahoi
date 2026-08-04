#!/usr/bin/env bash
#
# Guards the release-build constraint behind "panic path debug"
# (see `documents/todos.md`): the debug-only creation-site registry must never
# reach a release wasm binary.
#
# Nothing in the code enforces this — it rests entirely on the
# `#[cfg(debug_assertions)]` gates in `packages/ahoi-core/src/utils/location.rs`
# and `states/runtime/mod.rs`. Drop one and the release build silently starts
# tracking (and embedding) source locations again.
#
# Usage: scripts/check-wasm-release.sh

set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> building ahoi-wasm (release, wasm32-unknown-unknown)"
cargo build -p ahoi-wasm --target wasm32-unknown-unknown --release

WASM=$(cargo metadata --format-version 1 --no-deps \
    | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/wasm32-unknown-unknown/release/ahoi_wasm.wasm

if [ ! -f "$WASM" ]; then
    echo "FAIL: built wasm not found at $WASM" >&2
    exit 1
fi
echo "    $WASM ($(wc -c < "$WASM") bytes)"

echo "==> checking debug-only location registry is absent"
LEAKED=$(strings -n 4 "$WASM" \
    | grep -oE 'register_location|unregister_location|location_of' \
    | sort -u || true)

if [ -n "$LEAKED" ]; then
    echo "FAIL: debug-only location registry reached the release binary:" >&2
    echo "$LEAKED" | sed 's/^/    /' >&2
    echo >&2
    echo "A \`#[cfg(debug_assertions)]\` gate was dropped. Release builds must not" >&2
    echo "track creation sites — that is what keeps source paths out of the wasm." >&2
    exit 1
fi

echo "    ok — none present"
echo
echo "release wasm guard passed"
