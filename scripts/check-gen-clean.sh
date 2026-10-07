#!/usr/bin/env sh
# Fails if the committed web/src/gen files differ from what the Rust types generate.
# git diff ignores untracked files: the generated files must be committed.
set -eu
cd "$(dirname "$0")/.."
sh scripts/gen-types.sh
git diff --exit-code -- web/src/gen
echo "gen clean: web/src/gen matches the Rust types"
