#!/bin/sh
# Builds the WebAssembly bundles into web/src/wasm/pkg/<name>/.
#
#   sh scripts/build-wasm.sh            release build, optimized with wasm-opt
#   sh scripts/build-wasm.sh --dev      skips wasm-opt
#   sh scripts/build-wasm.sh --watch    builds, then builds again whenever a source file changes
#
# For each bundle: cargo builds the crate for wasm32 with SIMD, wasm-bindgen
# writes the JavaScript glue and the .d.ts, and wasm-opt shrinks the module.
set -eu
cd "$(dirname "$0")/.."

# One entry per bundle, "<crate>:<name>".
BUNDLES="offcut-wasm-core:core offcut-wasm-render:render"

TARGET=wasm32-unknown-unknown
# The features wasm-opt must accept. The first is what RUSTFLAGS turns on
# below; the others are on by default for this Rust target. If wasm-opt stops
# with a validation error that names a feature, add its flag here.
WASM_OPT_FEATURES="--enable-simd --enable-bulk-memory --enable-nontrapping-float-to-int"

dev=0
watch=0
for arg in "$@"; do
  case "$arg" in
    --dev) dev=1 ;;
    --watch) watch=1 ;;
    *)
      echo "build-wasm: unknown option: $arg" >&2
      exit 2
      ;;
  esac
done

# The wasm-bindgen CLI writes glue for one exact version of the crate. With
# any other version the module fails when it loads, so a mismatch stops here.
check_wasm_bindgen_version() {
  cli=$(wasm-bindgen --version | awk '{ print $2 }')
  crate=$(awk '$0 == "name = \"wasm-bindgen\"" { getline; gsub(/[^0-9.]/, ""); print; exit }' Cargo.lock)
  if [ -z "$crate" ]; then
    echo "build-wasm: Cargo.lock has no wasm-bindgen package" >&2
    return 1
  fi
  if [ "$cli" != "$crate" ]; then
    echo "build-wasm: the wasm-bindgen CLI is $cli, but Cargo.lock has the crate at $crate." >&2
    echo "build-wasm: install the matching CLI: cargo install wasm-bindgen-cli --version $crate --locked" >&2
    return 1
  fi
}

# Builds one bundle. Each step runs only if the one before it succeeded.
build_bundle() {
  crate=${1%%:*}
  name=${1##*:}
  out="web/src/wasm/pkg/$name"
  module="target/$TARGET/release/$(echo "$crate" | tr - _).wasm"

  RUSTFLAGS="-C target-feature=+simd128" \
    cargo build --target "$TARGET" --release -p "$crate" &&
    wasm-bindgen --target web --out-name "offcut_$name" --out-dir "$out" "$module" || return 1

  if [ "$dev" -eq 0 ]; then
    # shellcheck disable=SC2086  # the features are separate words on purpose
    wasm-opt -O3 $WASM_OPT_FEATURES "$out/offcut_${name}_bg.wasm" -o "$out/offcut_${name}_bg.wasm" || return 1
  fi
  echo "build-wasm: $out ($(wc -c < "$out/offcut_${name}_bg.wasm" | tr -d ' ') bytes)"
}

build_all() {
  check_wasm_bindgen_version || return 1
  for bundle in $BUNDLES; do
    build_bundle "$bundle" || return 1
  done
}

if [ "$watch" -eq 0 ]; then
  build_all
  exit
fi

# Watch mode. A failed build is reported and the watch goes on. The stamp
# file is touched before each build; a source file newer than it means
# something changed after that build started.
stamp="target/.build-wasm-stamp"
mkdir -p target
while :; do
  touch "$stamp"
  build_all || echo "build-wasm: the build failed; waiting for a change" >&2
  while [ -z "$(find crates Cargo.toml Cargo.lock -type f -newer "$stamp" \
    \( -name '*.rs' -o -name '*.toml' -o -name 'Cargo.lock' \) | head -n 1)" ]; do
    sleep 1
  done
done
