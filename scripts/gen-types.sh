#!/usr/bin/env sh
# Regenerates web/src/gen/domain.ts and web/src/gen/api.ts from the Rust types.
# Run from anywhere: sh scripts/gen-types.sh
set -eu
cd "$(dirname "$0")/.."
OFFCUT_GEN_OUT=web/src/gen/domain.ts cargo test -p offcut-types     write_typescript -- --ignored
OFFCUT_GEN_OUT=web/src/gen/api.ts    cargo test -p offcut-api-types write_typescript -- --ignored
