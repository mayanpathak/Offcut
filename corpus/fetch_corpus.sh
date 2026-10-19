#!/bin/sh
# corpus/fetch_corpus.sh - the corpus clips this machine has (v3implementation §17.6, D-88, D-112)
#
# Copies each clip of corpus/manifest.json into corpus/.clips/<id>.mp4 (git-ignored)
# and checks its SHA-256: a lent clip from $OFFCUT_CORPUS_DIR/<id>.<ext>, the
# founder's two from testclips/. Prints one line per clip, its id and `ok`,
# `missing` or `hash mismatch`; never a path, a file name or anything of a clip.
# No network. Exit 0 when at least one clip is ready, 1 when none is.
# Run by `pnpm corpus`, before the runner.
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
MANIFEST=$ROOT/corpus/manifest.json
CLIPS=$ROOT/corpus/.clips
hash() { sha256sum "$1" | cut -d' ' -f1; }

# The founder's two recordings are in testclips/ under their own names (D-99).
founder_file() {
  case "$1" in
    founder-01) echo "$ROOT/testclips/speech_scriptA_landscape_720p.mp4" ;;
    founder-02) echo "$ROOT/testclips/v3testclip.mp4" ;;
    *) echo "" ;;
  esac
}

# A lent clip keeps the extension it was lent with; the first file named <id>.<ext> is taken.
lent_file() {
  [ -n "${OFFCUT_CORPUS_DIR:-}" ] && [ -d "$OFFCUT_CORPUS_DIR" ] || { echo ""; return; }
  for candidate in "$OFFCUT_CORPUS_DIR/$1".*; do
    case "$candidate" in
      *.txt | *.part) continue ;;
    esac
    [ -f "$candidate" ] && { echo "$candidate"; return; }
  done
  echo ""
}

mkdir -p "$CLIPS"
# One line per clip: its id, SHA-256 and owner. The manifest is JSON, read with the Node pnpm runs on.
LIST=$(node -e '
  const m = JSON.parse(require("node:fs").readFileSync(process.argv[1], "utf8"));
  for (const c of m.clips) console.log(c.id + " " + c.sha256 + " " + c.owner);
' "$MANIFEST")

READY=0
while read -r id sha owner; do
  [ -n "$id" ] || continue
  dest=$CLIPS/$id.mp4
  # A copy made before, with the right hash, is kept as it is.
  if [ -f "$dest" ] && [ "$(hash "$dest")" = "$sha" ]; then
    echo "$id ok"
    READY=$((READY + 1))
    continue
  fi
  rm -f "$dest"
  if [ "$owner" = "founder" ]; then src=$(founder_file "$id"); else src=$(lent_file "$id"); fi
  if [ -z "$src" ] || [ ! -f "$src" ]; then
    echo "$id missing"
    continue
  fi
  cp "$src" "$dest.part"
  if [ "$(hash "$dest.part")" = "$sha" ]; then
    mv "$dest.part" "$dest"
    echo "$id ok"
    READY=$((READY + 1))
  else
    # The clip is skipped: a copy that is not the listed clip is never kept.
    rm -f "$dest.part"
    echo "$id hash mismatch"
  fi
done <<EOF
$LIST
EOF
[ "$READY" -gt 0 ] || { echo "no corpus clip is ready" >&2; exit 1; }
