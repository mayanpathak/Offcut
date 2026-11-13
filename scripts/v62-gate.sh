#!/usr/bin/env bash
# The V6.2 gate (docs/v6.2/v6.2implementation.md D-499; v6.2buildguide.md
# Step 0.4, Step 0.9; the SAB's gate in coding-promptsv6.2.md). Run from the
# repository root before every commit: `bash scripts/v62-gate.sh`.
# Prints "gate: ok", or one line per failed check and exits 1. It never
# prints a secret: a key is read from .env by name into a variable and only
# searched for.

set -u

BASE=b73fea5
fails=0
fail() {
  echo "FAIL: $*"
  fails=$((fails + 1))
}

cd "$(git rev-parse --show-toplevel)" || exit 2

# 1. The branch.
branch=$(git branch --show-current)
[ "$branch" = "v6.2-build" ] || fail "branch is '$branch', not v6.2-build"

# 2. No temporary zz- or zz_ file, tracked or not.
zz=$(git status --porcelain --untracked-files=all | grep -cE '(^|/| )zz[-_]')
[ "$zz" -eq 0 ] || fail "$zz zz-/zz_ file(s) in the tree"

# 3. No key in any tracked or untracked (not ignored) file: by pattern, and by
#    the exact values of the keys in .env, read by name and never printed. A
#    value under 20 characters is a local placeholder (the local .env's
#    TRANSCRIBE_API_KEY is one), not a key, and is not searched for.
git grep -qE 'AIza[0-9A-Za-z_-]{35}' && fail "a Google API key pattern is in the tree"
git grep -qE 'gsk_[A-Za-z0-9]{20}' && fail "a Groq key pattern is in the tree"
git grep -qE '(^|[^A-Za-z0-9])sk-[0-9a-f]{32}' && fail "a DeepSeek key pattern is in the tree"
# PLAN_API_KEY is DeepSeek's key since P189 (D-500, D-465); PLAN_API_KEY2, its old name, is kept in the list in case a file still has it.
for name in PLAN_API_KEY PLAN_API_KEY2 TRANSCRIBE_API_KEY; do
  value=$(grep "^$name=" .env 2>/dev/null | head -n 1 | cut -d= -f2-)
  if [ "${#value}" -ge 20 ] && git grep -qF --untracked -e "$value" -- . ':!.env' ':!.env.deploy'; then
    fail "the value of $name appears in a file"
  fi
done
unset value

# 4. The keyed e2e build's variable is not left in the environment.
[ -z "${VITE_ENTITLEMENT_TEST_PUBLIC_KEY:-}" ] || fail "VITE_ENTITLEMENT_TEST_PUBLIC_KEY is set: unset it before committing"

# 5. Frozen paths (Step 0.4): no change against the base, staged or not, and no new file in them.
FROZEN=(
  fixtures/labeled fixtures/speech fixtures/cloud verify web/src/models
  server/migrations/0001_init.sql crates/offcut-mp4 crates/offcut-text
  crates/offcut-render/src/video_pass.rs web/src/workers/asr web/src/workers/media
  web/src/usecases/transcribe-cloud.ts web/src/usecases/transcribe-source.ts
  web/src/usecases/run-source-pipeline.ts web/src/usecases/import-source.ts
  web/src/persistence/sources-repo.ts
)
for path in $(git diff --name-only "$BASE" -- "${FROZEN[@]}"); do
  fail "frozen file changed: $path"
done
for path in $(git ls-files --others --exclude-standard -- "${FROZEN[@]}"); do
  fail "new file in a frozen path: $path"
done
# As read (P184, D-439 under D-446): offcut-detect matches every EventKind by
# name, and D-446's codes 4-8 live in its event_id.rs, so the five kinds
# cannot be added with the crate frozen. It is additive instead: arms may be
# added (check 7), no line removed, no file added; its tests prove it unchanged.
for path in $(git diff --name-only --diff-filter=AD "$BASE" -- crates/offcut-detect; git ls-files --others --exclude-standard -- crates/offcut-detect); do
  fail "file added or removed in offcut-detect: $path"
done

# 6. No golden or snapshot that exists at the base is modified or deleted.
for path in $(git diff --name-only --diff-filter=MD "$BASE" -- '*.snap' '*.png'); do
  fail "golden or snapshot of the base changed: $path"
done

# 7. The additive check (Step 0.4): a file in ADDITIVE may lose a line only if
#    ALLOW has a pattern for it. A prompt that must remove a line adds the
#    pattern here, with the D-number that allows it, in the same commit.
ADDITIVE=(
  crates/offcut-types/src/events.rs crates/offcut-types/src/plan.rs crates/offcut-types/src/edit.rs
  crates/offcut-dsp/src/chain.rs crates/offcut-dsp/src/program.rs crates/offcut-scene/src/easing.rs
  crates/offcut-wasm-render/src/session.rs crates/offcut-wasm-render/src/scene_api.rs
  crates/offcut-wasm-core/src/audio_api.rs web/src/workers/protocol.ts web/src/net/api-client.ts
  web/src/persistence/db.ts web/src/persistence/schema.ts web/src/config/features.ts
  crates/offcut-detect
)
# Each entry: "<file>|<extended regex matched against the removed line, without its leading ->"  # D-n
ALLOW=(
  # P184 (D-446): EventParams gains five arms; the test that pinned four tagged kinds now pins nine.
  'crates/offcut-types/src/events.rs|^ +assert_eq!\(ts\.matches\(r#""kind": "#\)\.count\(\), 4, "\{ts\}"\);$'
  # P185 (D-454, D-440 (2)): a valid hook is kept now; the test's "not applied" example becomes the music cue.
  'crates/offcut-wasm-render/src/scene_api.rs|^ +let hook = input\(\|v\| v\["plan"\]\["extras"\]\["hook"\] = json!\("Watch this"\)\);$'
  'crates/offcut-wasm-render/src/scene_api.rs|^ +let \(_, dropped\) = scene_of\(hook, &clip\(\)\)\.unwrap\(\);$'
  # P186 (D-448, D-449): catalog 2 is this build's own, so the test's newer catalog becomes 3.
  'crates/offcut-wasm-render/src/scene_api.rs|^ +refused\(input\(\|v\| v\["plan"\]\["catalog"\] = json!\(2\)\)\),$'
  # P192 (D-443, Step 0.4): featureFlags' one-line return of three flags becomes a return of five.
  'web/src/config/features.ts|^  return \{ cloud_transcribe: value\("cloud_transcribe"\), long_video: value\("long_video"\), high_res: value\("high_res"\) \};$'
  # P193 (Step 0.4): META_KEYS' one line gains `aiPlan`, the settings switch, and is written one key a line.
  'web/src/persistence/schema.ts|^export const META_KEYS = \{ anonId: "anonId", persistRequested: "persistRequested", transcription: "transcription" \} as const;$'
  # P206c (D-503, D-440 (23)): a model plan with a valid hook gains the hook item, so the extras case keeps two.
  'crates/offcut-wasm-render/src/scene_api.rs|^ +assert_eq!\(checked\["kept"\], json!\(1\)\);$'
)
for file in "${ADDITIVE[@]}"; do
  while IFS= read -r line; do
    [ -z "$line" ] && continue
    removed=${line#-}
    allowed=0
    for entry in "${ALLOW[@]+"${ALLOW[@]}"}"; do
      if [ "${entry%%|*}" = "$file" ] && printf '%s\n' "$removed" | grep -qE -- "${entry#*|}"; then
        allowed=1
        break
      fi
    done
    [ "$allowed" -eq 1 ] || fail "removed line not allowed in $file: $removed"
  done < <(git diff -U0 "$BASE" -- "$file" | grep -E '^-[^-]')
done

# 8. Nothing private is tracked.
private=$(git ls-files --cached | grep -E '(^|/)\.env|testclips|\.gen-local|dist-flags' || true)
[ -z "$private" ] || fail "a private file is tracked: $private"

# 9. Every source file is in TS §5, and within the line limit.
node scripts/check-file-tree.mjs > /dev/null 2>&1 || fail "check-file-tree.mjs: $(node scripts/check-file-tree.mjs 2>&1 | tail -n 1)"

# 10. D-440 (9): the release's only ignore is gone once P206 is committed.
if git log --oneline "$BASE"..HEAD | grep -q 'V6.2 P206'; then
  if git grep -q 'ignore = "V6.2' -- crates; then
    fail "an ignore marked V6.2 is left after P206"
  fi
fi

if [ "$fails" -eq 0 ]; then
  echo "gate: ok"
  exit 0
fi
echo "gate: $fails failure(s)"
exit 1
